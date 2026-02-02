package io.github.liyze09.nexus.chunk;

import io.github.liyze09.nexus.NexusBackend;
import io.github.liyze09.nexus.model.block.Mesh;
import io.github.liyze09.nexus.model.block.Model;
import io.github.liyze09.nexus.model.block.ModelManager;
import io.github.liyze09.nexus.model.block.VisibleFaces;
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
import net.minecraft.world.phys.shapes.CollisionContext;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.io.Closeable;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.util.ArrayList;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicReferenceArray;

public class NexusChunkBuilder implements Closeable {
    final Arena arena = Arena.ofShared();
    final Map<ChunkPos, LevelChunk> loadedChunks;
    final Map<ChunkPos, BuiltChunk> builtChunks;
    final NexusBackend backend;
    ClientLevel world;
    AtomicReferenceArray<LevelChunk> chunks;
    Camera camera;
    CollisionContext collisionContext = CollisionContext.empty();

    public NexusChunkBuilder(@NotNull ClientLevel world, @NotNull NexusBackend backend) {
        this.world = world;
        this.backend = backend;
        this.chunks = world.getChunkSource().storage.chunks;
        this.loadedChunks = new ConcurrentHashMap<>(chunks.length());
        this.builtChunks = new ConcurrentHashMap<>(chunks.length());
        rebuild0(chunks, loadedChunks);
    }

    private static void rebuild0(@NotNull AtomicReferenceArray<LevelChunk> chunks, Map<ChunkPos, LevelChunk> loadedChunks) {
        for (int i = 0; i < chunks.length(); i++) {
            LevelChunk chunk = chunks.get(i);
            if (chunk != null) {
                loadedChunks.put(chunk.getPos(), chunk);
            }
        }
    }

    public void setCamera(@Nullable Camera camera) {
        this.camera = camera;
        this.collisionContext = camera == null ? CollisionContext.empty() : CollisionContext.of(camera.entity());
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
        if (chunk == null) {
            return;
        }
        ModelManager modelManager = ModelManager.getInstance();
        LayeredBlockGetter blockGetter = new LayeredBlockGetter(world, chunk);
        ArrayList<Float> verticesList = new ArrayList<>();
        ArrayList<Integer> indicesList = new ArrayList<>();

        int vertexOffset = 0;

        for (int i = 0; i < chunk.getSections().length; i++) {
            LevelChunkSection section = chunk.getSections()[i];
            if (section.hasOnlyAir()) {
                continue;
            }
            blockGetter.setSection(section, i);
            BlockPos blockPos0 = SectionPos.of(chunkPos, i).origin();
            BlockPos blockPos1 = blockPos0.offset(15, 15, 15);
            for (BlockPos pos : BlockPos.betweenClosed(blockPos0, blockPos1)) {
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
                if (mesh.isPresent()) {
                    Mesh m = mesh.get();
                    // Add vertices
                    for (float vertex : m.vertices) {
                        verticesList.add(vertex);
                    }
                    // Add indices with offset
                    for (int index : m.indices) {
                        indicesList.add(index + vertexOffset);
                    }
                    // Update vertex offset for next mesh
                    vertexOffset += m.vertices.length / 3; // Each vertex has 3 components (x, y, z)
                }
            }
        }

        // Convert lists to arrays
        float[] vertices = new float[verticesList.size()];
        for (int j = 0; j < verticesList.size(); j++) {
            vertices[j] = verticesList.get(j);
        }
        int[] indices = new int[indicesList.size()];
        for (int j = 0; j < indicesList.size(); j++) {
            indices[j] = indicesList.get(j);
        }

        // Upload mesh data to Rust backend
        if (vertices.length > 0 && indices.length > 0) {
            backend.uploadChunkMesh(chunkPos.x, chunkPos.z, vertices, indices);
        }

        // Still create a built chunk for compatibility (empty for now)
        BuiltChunk builtChunk = new BuiltChunk();
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
