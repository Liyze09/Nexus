package io.github.liyze09.nexus.mixin.client.accessor;

import com.mojang.blaze3d.opengl.GlTexture;
import it.unimi.dsi.fastutil.ints.Int2IntMap;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(GlTexture.class)
public interface GlTextureAccessor {
    @Accessor
    int getViews();

    @Accessor
    int getFirstFboId();

    @Accessor
    Int2IntMap getFboCache();
}
