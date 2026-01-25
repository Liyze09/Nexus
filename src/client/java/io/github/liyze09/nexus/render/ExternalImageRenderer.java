package io.github.liyze09.nexus.render;

import com.mojang.blaze3d.opengl.GlTexture;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.GpuTextureView;
import com.mojang.blaze3d.textures.TextureFormat;
import org.jspecify.annotations.NonNull;
import org.lwjgl.opengl.EXTSemaphore;

import static org.lwjgl.opengl.EXTMemoryObject.*;
import static org.lwjgl.opengl.EXTMemoryObjectWin32.glImportMemoryWin32HandleEXT;
import static org.lwjgl.opengl.EXTSemaphore.*;
import static org.lwjgl.opengl.EXTSemaphoreWin32.*;
import static org.lwjgl.opengl.GL30C.*;

public class ExternalImageRenderer {
    private final long memoryHandle;
    private final long size;
    private final long readyHandle;
    private final long completeHandle;
    public GlTexture blaze3dTexture;
    public GpuTextureView blaze3dTextureView;
    private int glTextureId;
    private int glTextureMemoryId;
    private int glReadySemaphoreId;
    private int glCompleteSemaphoreId;
    private int textureWidth = 0;
    private int textureHeight = 0;

    public ExternalImageRenderer(long readyHandle, long completeHandle, long memoryHandle, int width, int height, long size) {
        this.readyHandle = readyHandle;
        this.completeHandle = completeHandle;
        this.size = size;
        this.textureWidth = width;
        this.textureHeight = height;
        this.memoryHandle = memoryHandle;
        importSemaphores();
        importTextureMemory();
    }

    private void importSemaphores() {
        glReadySemaphoreId = EXTSemaphore.glGenSemaphoresEXT();
        glImportSemaphoreWin32HandleEXT(glReadySemaphoreId,
                GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                readyHandle);

        glCompleteSemaphoreId = EXTSemaphore.glGenSemaphoresEXT();
        glImportSemaphoreWin32HandleEXT(glCompleteSemaphoreId,
                GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                completeHandle);
    }

    private void importTextureMemory() {
        blaze3dTexture = (GlTexture) RenderSystem.getDevice().createTexture((String) null, 15, TextureFormat.RGBA8, textureWidth, textureHeight, 1, 1);
        blaze3dTextureView = RenderSystem.getDevice().createTextureView(blaze3dTexture);
        glTextureId = blaze3dTexture.glId();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, glTextureId);

        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

        int memoryObjectId = glCreateMemoryObjectsEXT();
        glImportMemoryWin32HandleEXT(memoryObjectId,
                size,
                GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                memoryHandle);

        glTextureStorageMem2DEXT(glTextureId,
                1,
                GL_RGBA8,
                textureWidth,
                textureHeight,
                memoryObjectId,
                0);

        glTextureMemoryId = memoryObjectId;
    }

    public void render(@NonNull Runnable external) {
        signalComplete();
        glFlush();
        external.run();
        waitForReadySignal();
        glFlush();
    }

    private void waitForReadySignal() {
        int[] textures = {glTextureId};
        int[] srcLayouts = {GL_LAYOUT_COLOR_ATTACHMENT_EXT};
        glWaitSemaphoreEXT(glReadySemaphoreId,
                new int[0],
                textures,
                srcLayouts
        );
    }

    private void signalComplete() {
        int[] textures = {glTextureId};
        int[] dstLayouts = {GL_LAYOUT_SHADER_READ_ONLY_EXT};

        glSignalSemaphoreEXT(glCompleteSemaphoreId,
                new int[0],
                textures,
                dstLayouts
        );
    }

    public void cleanup() {
        if (glTextureId != 0) {
            blaze3dTexture.close();
            blaze3dTextureView.close();
        }

        if (glTextureMemoryId != 0) {
            glDeleteMemoryObjectsEXT(glTextureMemoryId);
        }

        if (glReadySemaphoreId != 0) {
            glDeleteSemaphoresEXT(glReadySemaphoreId);
        }

        if (glCompleteSemaphoreId != 0) {
            glDeleteSemaphoresEXT(glCompleteSemaphoreId);
        }
    }
}