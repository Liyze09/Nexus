package io.github.liyze09.nexus.resource;

import com.mojang.blaze3d.opengl.GlStateManager;
import com.mojang.blaze3d.opengl.GlTexture;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.TextureFormat;
import io.github.liyze09.nexus.NexusBackend;
import io.github.liyze09.nexus.mixin.client.accessor.GlTextureAccessor;

import static org.lwjgl.opengl.EXTMemoryObject.*;
import static org.lwjgl.opengl.EXTMemoryObjectWin32.*;
import static org.lwjgl.opengl.GL30C.*;

public class ExternalGlTexture extends GlTexture {
    public final GlTexture texture;
    public final long handle;
    private int memoryObjectId;

    public ExternalGlTexture(@Usage int i, String string, TextureFormat textureFormat, int width, int height, int mipLevels, NexusBackend backend) {
        // Generate a new OpenGL texture ID
        GlTexture texture = (GlTexture) RenderSystem.getDevice().createTexture(string, i, textureFormat, width, height, 1, mipLevels);
        this.texture = texture;
        int glId = texture.glId();
        super(i, string, textureFormat, width, height, 1, mipLevels, glId);

        try {
            // Acquire Vulkan texture with full mip chain
            long handle = backend.acquireVulkanTexture(width, height, mipLevels);
            this.handle = handle;
            if (handle == 0) {
                throw new RuntimeException("Failed to acquire Vulkan texture handle");
            }

            // Get actual total memory size from Vulkan
            long totalSize = backend.getVulkanTextureSize(handle);
            if (totalSize <= 0) {
                throw new RuntimeException("Failed to get Vulkan texture size");
            }

            // Import the single memory object
            memoryObjectId = glCreateMemoryObjectsEXT();
            glImportMemoryWin32HandleEXT(memoryObjectId,
                    totalSize,
                    GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                    handle);

            // Bind texture storage with all mip levels
            // Note: The offset parameter in glTextureStorageMem2DEXT is for the entire texture
            // Each mip level will be automatically placed at the correct offset by the driver
            glTextureStorageMem2DEXT(glId,
                    mipLevels,
                    GL_RGBA8, // Hardcoded as per user choice
                    width,
                    height,
                    memoryObjectId,
                    0);

            // Set texture parameters
            glBindTexture(GL_TEXTURE_2D, glId);
            if (mipLevels > 1) {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
            } else {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            }
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
            glBindTexture(GL_TEXTURE_2D, 0);

        } catch (Exception e) {
            // Cleanup on error
            if (memoryObjectId != 0) {
                glDeleteMemoryObjectsEXT(memoryObjectId);
                memoryObjectId = 0;
            }
            throw e;
        }
    }

    @Override
    public void close() {
        if (!super.closed) {
            super.closed = true;
            var accessor = (GlTextureAccessor) this;
            if (accessor.getViews() == 0) {
                if (memoryObjectId != 0) {
                    glDeleteMemoryObjectsEXT(memoryObjectId);
                    GlStateManager._deleteTexture(this.id);
                    if (accessor.getFirstFboId() != -1) {
                        GlStateManager._glDeleteFramebuffers(accessor.getFirstFboId());
                    }

                    if (accessor.getFboCache() != null) {
                        for (int i : accessor.getFboCache().values()) {
                            GlStateManager._glDeleteFramebuffers(i);
                        }
                    }
                    memoryObjectId = 0;
                }
            }
        }
    }
}
