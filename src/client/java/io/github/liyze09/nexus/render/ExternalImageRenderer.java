package io.github.liyze09.nexus.render;

import org.lwjgl.opengl.*;
import org.lwjgl.system.*;

import java.nio.FloatBuffer;

import static com.mojang.blaze3d.opengl.GlConst.GL_TRUE;
import static org.lwjgl.glfw.GLFW.*;
import static org.lwjgl.opengl.EXTMemoryObject.*;
import static org.lwjgl.opengl.EXTMemoryObjectWin32.glImportMemoryWin32HandleEXT;
import static org.lwjgl.opengl.EXTSemaphore.*;
import static org.lwjgl.opengl.EXTSemaphoreWin32.*;

public class ExternalImageRenderer {
    private int vaoId;
    private int vboId;
    private int shaderProgramId;
    private int glTextureId;

    private int glReadySemaphoreId;
    private int glCompleteSemaphoreId;

    private long memoryHandle;
    private long size;
    private final long readyHandle;
    private final long completeHandle;

    private int textureWidth = 0;
    private int textureHeight = 0;

    public ExternalImageRenderer(long readyHandle, long completeHandle) {
        this.readyHandle = readyHandle;
        this.completeHandle = completeHandle;
        glfwWindowHint(GLFW_OPENGL_DEBUG_CONTEXT, GL_TRUE);
        compileShaders();
        importSemaphores();
    }

    private void createFullScreenTriangle() {
        float[] vertices = {
                -1.0f, -1.0f, 0.0f, 0.0f,
                3.0f, -1.0f, 2.0f, 0.0f,
                -1.0f,  3.0f, 0.0f, 2.0f
        };

        vaoId = GL30.glGenVertexArrays();
        GL30.glBindVertexArray(vaoId);

        FloatBuffer verticesBuffer = MemoryUtil.memAllocFloat(vertices.length);
        verticesBuffer.put(vertices).flip();

        vboId = GL30.glGenBuffers();
        GL30.glBindBuffer(GL30.GL_ARRAY_BUFFER, vboId);
        GL30.glBufferData(GL30.GL_ARRAY_BUFFER, verticesBuffer, GL30.GL_STATIC_DRAW);
        MemoryUtil.memFree(verticesBuffer);

        GL30.glVertexAttribPointer(0, 2, GL30.GL_FLOAT, false, 4 * Float.BYTES, 0);
        GL30.glEnableVertexAttribArray(0);

        GL30.glVertexAttribPointer(1, 2, GL30.GL_FLOAT, false, 4 * Float.BYTES, 2 * Float.BYTES);
        GL30.glEnableVertexAttribArray(1);

        GL30.glBindVertexArray(0);
    }

    private int compileShader(int type, String source) {
        int shader = GL20.glCreateShader(type);
        GL20.glShaderSource(shader, source);
        GL20.glCompileShader(shader);

        if (GL20.glGetShaderi(shader, GL20.GL_COMPILE_STATUS) == GL11.GL_FALSE) {
            String log = GL20.glGetShaderInfoLog(shader, 512);
            GL20.glDeleteShader(shader);
            throw new RuntimeException("Shader compilation failed: " + log);
        }

        return shader;
    }

    private void compileShaders() {
        String vertexShaderSource =
                """
                        #version 330
                        layout(location = 0) in vec2 aPos;
                        layout(location = 1) in vec2 aTexCoord;
                        out vec2 texCoord;
                        void main() {
                            gl_Position = vec4(aPos, 0.0, 1.0);
                            texCoord = aTexCoord;
                        }""";

        String fragmentShaderSource =
                """
                        #version 330
                        in vec2 texCoord;
                        out vec4 FragColor;
                        uniform sampler2D uTexture;
                        void main() {
                            FragColor = texture(uTexture, texCoord);
                        }""";

        int vertexShader = compileShader(GL20.GL_VERTEX_SHADER, vertexShaderSource);
        int fragmentShader = compileShader(GL20.GL_FRAGMENT_SHADER, fragmentShaderSource);

        shaderProgramId = GL20.glCreateProgram();
        GL20.glAttachShader(shaderProgramId, vertexShader);
        GL20.glAttachShader(shaderProgramId, fragmentShader);
        GL20.glLinkProgram(shaderProgramId);

        if (GL20.glGetProgrami(shaderProgramId, GL20.GL_LINK_STATUS) == GL11.GL_FALSE) {
            String log = GL20.glGetProgramInfoLog(shaderProgramId, 512);
            throw new RuntimeException("Shader program linking failed: " + log);
        }

        GL20.glDeleteShader(vertexShader);
        GL20.glDeleteShader(fragmentShader);
    }

    public void update(long memoryHandle, int width, int height, long size) {
        if (this.memoryHandle == memoryHandle && this.textureWidth == width && this.textureHeight == height) {
            return;
        }
        this.memoryHandle = memoryHandle;
        this.textureWidth = width;
        this.textureHeight = height;
        this.size = size;
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
        glTextureId = GL11.glGenTextures();
        GL13.glActiveTexture(GL13.GL_TEXTURE0);
        GL11.glBindTexture(GL11.GL_TEXTURE_2D, glTextureId);

        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MIN_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MAG_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_S, GL12.GL_CLAMP_TO_EDGE);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_T, GL12.GL_CLAMP_TO_EDGE);

        int memoryObjectId = glCreateMemoryObjectsEXT();
        glImportMemoryWin32HandleEXT(memoryObjectId,
                size,
                GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                memoryHandle);

        glTextureStorageMem2DEXT(glTextureId,
                1,
                GL30.GL_RGBA8,
                textureWidth,
                textureHeight,
                memoryObjectId,
                0);
        glDeleteMemoryObjectsEXT(memoryObjectId);
    }

    public void render() {
        waitForReadySignal();

        executeOpenGLRender();

        signalComplete();
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

    private void executeOpenGLRender() {
        GL20.glUseProgram(shaderProgramId);

        int textureLoc = GL20.glGetUniformLocation(shaderProgramId, "uTexture");
        GL20.glUniform1i(textureLoc, 0);

        GL13.glActiveTexture(GL13.GL_TEXTURE0);
        GL11.glBindTexture(GL11.GL_TEXTURE_2D, glTextureId);

        GL30.glBindVertexArray(vaoId);
        GL11.glDrawArrays(GL11.GL_TRIANGLES, 0, 3);
        GL30.glBindVertexArray(0);
        GL20.glUseProgram(0);
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
            GL11.glDeleteTextures(glTextureId);
        }

        if (glReadySemaphoreId != 0) {
            glDeleteSemaphoresEXT(glReadySemaphoreId);
        }

        if (glCompleteSemaphoreId != 0) {
            glDeleteSemaphoresEXT(glCompleteSemaphoreId);
        }

        if (shaderProgramId != 0) {
            GL20.glDeleteProgram(shaderProgramId);
        }

        if (vboId != 0) {
            GL15.glDeleteBuffers(vboId);
        }

        if (vaoId != 0) {
            GL30.glDeleteVertexArrays(vaoId);
        }
    }

    public int getTextureId() {
        return glTextureId;
    }

    public int getReadySemaphoreId() {
        return glReadySemaphoreId;
    }

    public int getCompleteSemaphoreId() {
        return glCompleteSemaphoreId;
    }
}