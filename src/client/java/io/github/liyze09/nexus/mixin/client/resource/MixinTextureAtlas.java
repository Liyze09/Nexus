package io.github.liyze09.nexus.mixin.client.resource;

import com.mojang.blaze3d.systems.GpuDevice;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.TextureFormat;
import io.github.liyze09.nexus.NexusClientMain;
import io.github.liyze09.nexus.render.NexusWorldRenderer;
import io.github.liyze09.nexus.resource.ExternalGlTexture;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.*;
import net.minecraft.resources.Identifier;
import org.jspecify.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;
import java.util.function.Supplier;

@Mixin(TextureAtlas.class)
public abstract class MixinTextureAtlas extends AbstractTexture {
    @Final
    @Shadow
    private Identifier location;

    @Redirect(
            method = "createTexture",
            at = @At(
                    value = "INVOKE",
                    target = "Lcom/mojang/blaze3d/systems/GpuDevice;createTexture(Ljava/util/function/Supplier;ILcom/mojang/blaze3d/textures/TextureFormat;IIII)Lcom/mojang/blaze3d/textures/GpuTexture;"

            )
    )
    public GpuTexture nexus$redirectTexture(GpuDevice instance, @Nullable Supplier<String> stringSupplier, @GpuTexture.Usage int usage, TextureFormat textureFormat, int i, int j, int k, int l) {
        if (!NexusClientMain.getNexusRenderer().backend.sharedTextureManager.textureManagedByVulkan.contains(location)) {
            return instance.createTexture(stringSupplier, usage, textureFormat, i, j, k, l);
        }
        var texture = new ExternalGlTexture(usage, location.toString(), textureFormat, i, j, l, ((NexusWorldRenderer) Minecraft.getInstance().levelRenderer).backend);
        NexusClientMain.getNexusRenderer().backend.sharedTextureManager.addTexture(location, texture);
        return texture;
    }

    @Inject(method = "upload", at = @At("RETURN"))
    public void nexus$sync(SpriteLoader.Preparations preparations, CallbackInfo ci) {
        var renderer = NexusClientMain.getNexusRenderer();
        if (!renderer.backend.sharedTextureManager.textureManagedByVulkan.contains(location)) {
            return;
        }
        String atlasName = location.toString();
        Map<Identifier, TextureAtlasSprite> map = preparations.regions();
        ExternalGlTexture texture = (ExternalGlTexture) this.getTexture();
        long id = texture.handle;

        // Prepare arrays for sprite data
        int size = map.size();
        String[] spriteNames = new String[size];
        int[] spriteX = new int[size];
        int[] spriteY = new int[size];
        int[] spriteWidth = new int[size];
        int[] spriteHeight = new int[size];
        float[] spriteU0 = new float[size];
        float[] spriteV0 = new float[size];
        float[] spriteU1 = new float[size];
        float[] spriteV1 = new float[size];

        int index = 0;
        for (Map.Entry<Identifier, TextureAtlasSprite> entry : map.entrySet()) {
            TextureAtlasSprite sprite = entry.getValue();
            spriteNames[index] = entry.getKey().toString();
            spriteX[index] = sprite.getX();
            spriteY[index] = sprite.getY();

            // Get width and height from sprite contents
            SpriteContents contents = sprite.contents();
            spriteWidth[index] = contents.width();
            spriteHeight[index] = contents.height();

            // Get UV coordinates
            spriteU0[index] = sprite.getU0();
            spriteV0[index] = sprite.getV0();
            spriteU1[index] = sprite.getU1();
            spriteV1[index] = sprite.getV1();

            index++;
        }

        // Call native function to sync atlas data
        renderer.backend.syncAtlas(
            id,
            atlasName,
            spriteNames,
            spriteX, spriteY,
            spriteWidth, spriteHeight,
            spriteU0, spriteV0,
            spriteU1, spriteV1
        );
    }
}
