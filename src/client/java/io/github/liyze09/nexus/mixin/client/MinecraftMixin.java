package io.github.liyze09.nexus.mixin.client;

import io.github.liyze09.nexus.render.NexusWorldRenderer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LevelRenderer;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Minecraft.class)
public abstract class MinecraftMixin {
    @Redirect(
            method = "<init>",
            at = @At(value = "FIELD",
                    target = "Lnet/minecraft/client/Minecraft;levelRenderer:Lnet/minecraft/client/renderer/LevelRenderer;",
                    opcode = Opcodes.PUTFIELD
            ))
    public void nexus$init(Minecraft instance, LevelRenderer value) {
        instance.levelRenderer = new NexusWorldRenderer(instance,
                instance.getEntityRenderDispatcher(),
                instance.getBlockEntityRenderDispatcher(),
                instance.renderBuffers,
                instance.gameRenderer.getLevelRenderState(),
                instance.gameRenderer.getFeatureRenderDispatcher()
        );
    }
}
