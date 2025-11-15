package io.github.liyze09.nexus.mixin.client;

import net.minecraft.client.renderer.LevelRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(LevelRenderer.class)
public abstract class LevelRendererMixin {
    @Inject(method = "<init>", at = @At("CTOR_HEAD"), cancellable = true)
    private static void init(CallbackInfo ci) {
        ci.cancel();
    }
}
