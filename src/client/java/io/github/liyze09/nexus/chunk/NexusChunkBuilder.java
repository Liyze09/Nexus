package io.github.liyze09.nexus.chunk;

import java.io.Closeable;
import java.lang.foreign.Arena;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicReferenceArray;

import io.github.liyze09.nexus.NexusClientMain;
import io.github.liyze09.nexus.model.block.*;
import io.github.liyze09.nexus.utils.LayeredBlockGetter;
import io.github.liyze09.nexus.utils.VkPos;
import net.minecraft.client.Camera;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.CollisionContext;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

public class NexusChunkBuilder implements Closeable {
    ClientLevel world;
    final Arena arena = Arena.ofShared();
    AtomicReferenceArray<LevelChunk> chunks;
    final Map<ChunkPos, LevelChunk> loadedChunks;
    final Map<ChunkPos, BuiltChunk> builtChunks;
    Camera camera;
    CollisionContext collisionContext = CollisionContext.empty();
    public NexusChunkBuilder(@NotNull ClientLevel world) {
        this.world = world;
        this.chunks = world.getChunkSource().storage.chunks;
        this.loadedChunks = new ConcurrentHashMap<>(chunks.length());
        this.builtChunks = new ConcurrentHashMap<>(chunks.length());
        rebuild0(chunks, loadedChunks);
    }

    public void setCamera(@Nullable Camera camera) {
        this.camera = camera;
        this.collisionContext = camera == null ? CollisionContext.empty() : CollisionContext.of(camera.entity());
    }

    private static void rebuild0(@NotNull AtomicReferenceArray<LevelChunk> chunks, Map<ChunkPos, LevelChunk> loadedChunks) {
        for (int i = 0; i < chunks.length(); i++) {
            LevelChunk chunk = chunks.get(i);
            if (chunk != null) {
                loadedChunks.put(chunk.getPos(), chunk);
            }
        }
    }

    public void load(LevelChunk chunk) {
        if (chunk != null) {
            loadedChunks.put(chunk.getPos(), chunk);
        }
    }

    public void unload(ChunkPos chunkPos) {
        loadedChunks.remove(chunkPos);
        builtChunks.remove(chunkPos);
    }

    public void rebuild(@NotNull ClientLevel world) {
        this.loadedChunks.clear();
        this.builtChunks.clear();
        rebuild0(world.getChunkSource().storage.chunks, this.loadedChunks);
    }

    public boolean isSectionBuilt(BlockPos blockPos) {
        LevelChunk chunk = loadedChunks.get(new ChunkPos(blockPos));
        if (chunk != null) {
            return builtChunks.containsKey(chunk.getPos());
        }
        return false;
    }

    public void build(ChunkPos chunkPos) {
        LevelChunk chunk = loadedChunks.get(chunkPos);
        if (chunk == null) {return;}
        ModelManager modelManager = ModelManager.getInstance();
        BuiltChunk builtChunk = new BuiltChunk();
        LayeredBlockGetter blockGetter = new LayeredBlockGetter(world, chunk);
        ArrayList<Mesh> meshes = new ArrayList<>();
        ArrayList<AABB> aabbs = new ArrayList<>();
        for (int i = 0; i < chunk.getSections().length; i++) {
            LevelChunkSection section = chunk.getSections()[i];
            if (section.hasOnlyAir()) {
                continue;
            }
            blockGetter.setSection(section, i);
            BlockPos blockPos0 = SectionPos.of(chunkPos, i).origin();
            BlockPos blockPos1 = blockPos0.offset(15, 15, 15);
            for (BlockPos pos: BlockPos.betweenClosed(blockPos0, blockPos1)) {
                BlockState state = section.getBlockState(pos.getX() - blockPos0.getX(), pos.getY() - blockPos0.getY(), pos.getZ() - blockPos0.getZ());
                if (state.isAir()) {
                    continue;
                }
                VisibleFaces faces = new VisibleFaces();
                if (blockGetter.getBlockState(pos.above()).canOcclude()) {
                    faces.up = false;
                }
                if (blockGetter.getBlockState(pos.below()).canOcclude()) {
                    faces.down = false;
                }
                if (blockGetter.getBlockState(pos.north()).canOcclude()) {
                    faces.north = false;
                }
                if (blockGetter.getBlockState(pos.east()).canOcclude()) {
                    faces.east = false;
                }
                if (blockGetter.getBlockState(pos.south()).canOcclude()) {
                    faces.south = false;
                }
                if (blockGetter.getBlockState(pos.west()).canOcclude()) {
                    faces.west = false;
                }
                Model model = modelManager.getModel(state);
                var pos1 = new VkPos(
                        pos.getX() & 15,
                        pos.getY(),
                        pos.getZ() & 15
                );
                Optional<Mesh> mesh = model.getMesh(faces, pos1);
                if (!mesh.isEmpty()) {
                    meshes.add(mesh.get());
                }
                var aabb = model.getAABB(faces, pos1, NexusClientMain.config.parallax_depth);
                aabbs.addAll(aabb);
            }
        }
        
        if (aabbs.size() != 0) {
            var layout = MemoryLayout.sequenceLayout(
                    aabbs.size() * 6,
                    ValueLayout.JAVA_FLOAT
            );
            MemorySegment segment_aabb = arena.allocate(layout);
            for (int j = 0; j < aabbs.size(); j++) {
                var aabb1 = aabbs.get(j);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 0, (float) aabb1.minX);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 1, (float) aabb1.minY);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 2, (float) aabb1.minZ);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 3, (float) aabb1.maxX);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 4, (float) aabb1.maxY);
                segment_aabb.setAtIndex(ValueLayout.JAVA_FLOAT, j * 6 + 5, (float) aabb1.maxZ);
            }
            builtChunk.segmentAABB = segment_aabb;
        }
        
        builtChunks.put(chunkPos, builtChunk);
    }

    public void close() {
        arena.close();
    }

    public class BuiltChunk {
        MemorySegment segmentAABB = MemorySegment.NULL;
        public BuiltChunk() {

        }
    }
}
