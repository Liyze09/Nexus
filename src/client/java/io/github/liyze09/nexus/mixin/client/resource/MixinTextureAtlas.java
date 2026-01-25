package io.github.liyze09.nexus.mixin.client.resource;

import com.mojang.blaze3d.systems.GpuDevice;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.TextureFormat;
import io.github.liyze09.nexus.render.NexusWorldRender;
import io.github.liyze09.nexus.resource.ExternalGlTexture;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.TextureAtlas;
import org.jspecify.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.util.function.Supplier;

@Mixin(TextureAtlas.class)
public class MixinTextureAtlas {
    @Redirect(
            method = "createTexture",
            at = @At(
                    value = "INVOKE",
                    target = "Lcom/mojang/blaze3d/systems/GpuDevice;createTexture(Ljava/util/function/Supplier;ILcom/mojang/blaze3d/textures/TextureFormat;IIII)Lcom/mojang/blaze3d/textures/GpuTexture;"

            )
    )
    public GpuTexture nexus$redirectTexture(GpuDevice instance, @Nullable Supplier<String> stringSupplier, @GpuTexture.Usage int usage, TextureFormat textureFormat, int i, int j, int k, int l) {
        return new ExternalGlTexture(usage, stringSupplier == null ? "" : stringSupplier.get(), textureFormat, i, j, l, ((NexusWorldRender) Minecraft.getInstance().levelRenderer).backend);
    }
}
