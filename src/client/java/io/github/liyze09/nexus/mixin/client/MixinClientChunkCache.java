package io.github.liyze09.nexus.mixin.client;

import io.github.liyze09.nexus.render.NexusWorldRender;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientChunkCache;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Map;
import java.util.Objects;
import java.util.function.Consumer;

@Mixin(ClientChunkCache.class)
public class MixinClientChunkCache {
    @Inject(method = "updateViewRadius", at = @At("TAIL"))
    public void nexus$onViewRadiusChanged(int i, CallbackInfo ci) {
        NexusWorldRender render = (NexusWorldRender) Minecraft.getInstance().levelRenderer;
        if (render.builder != null && render.getWorld() != null) {
            render.builder.rebuild(Objects.requireNonNull(render.getWorld()));
        } else {
            render.builder.close();
            render.builder = null;
        }
    }

    @Inject(method = "replaceWithPacketData", at = @At("RETURN"))
    public void nexus$onUpdateChunk(int i, int j, FriendlyByteBuf friendlyByteBuf, Map<Heightmap.Types, long[]> map, Consumer<ClientboundLevelChunkPacketData.BlockEntityTagOutput> consumer, CallbackInfoReturnable<LevelChunk> cir) {
        NexusWorldRender render = (NexusWorldRender) Minecraft.getInstance().levelRenderer;
        if (render.builder != null) {
            render.builder.load(cir.getReturnValue());
        }
    }

    @Inject(method = "drop", at = @At("HEAD"))
    public void nexus$onChunkUnload(ChunkPos pos, CallbackInfo ci) {
        NexusWorldRender render = (NexusWorldRender) Minecraft.getInstance().levelRenderer;
        if (render.builder != null) {
            render.builder.unload(pos);
        }
    }
}
