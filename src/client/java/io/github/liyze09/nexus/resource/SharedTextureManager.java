package io.github.liyze09.nexus.resource;

import com.mojang.blaze3d.textures.GpuTextureView;
import net.minecraft.resources.Identifier;

import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

public class SharedTextureManager {
    private final ConcurrentHashMap<Identifier, GpuTextureView[]> textures = new ConcurrentHashMap<>();

    public void addTexture(Identifier identifier, GpuTextureView[] texture) {
        textures.put(identifier, texture);
    }

    public Map<Identifier, long[]> genExportedTextureIds() {
        var ret = new HashMap<Identifier, long[]>();
        textures.forEach((identifier, texture) ->
                ret.put(identifier, Arrays.stream(texture).mapToLong(
                        textureView ->
                                ((ExternalGlTexture) textureView.texture()).handle
                ).toArray())
        );
        return ret;
    }
}
