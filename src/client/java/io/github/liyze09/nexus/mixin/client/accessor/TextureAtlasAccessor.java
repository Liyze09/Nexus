package io.github.liyze09.nexus.mixin.client.accessor;

import com.mojang.blaze3d.textures.GpuTextureView;
import net.minecraft.client.renderer.texture.TextureAtlas;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(TextureAtlas.class)
public interface TextureAtlasAccessor {
    @Accessor
    GpuTextureView[] getMipViews();
}
