package io.github.liyze09.nexus.render;

import static org.lwjgl.opengl.EXTMemoryObject.*;
import static org.lwjgl.opengl.EXTMemoryObjectWin32.*;
import static org.lwjgl.opengl.GL30.*;

import java.nio.FloatBuffer;
import java.nio.IntBuffer;

import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

public class ExternalImageRender {
    private int shaderProgram, quadVAO, quadVBO;
    private int currentTextureId = 0;
    private boolean initialized = false;
    
    private void initialize() {
        if (initialized) return;
        
        initShaders();
        initQuad();
        initialized = true;
    }
    
    public void render(long handle, int width, int height) {
        initialize();

        int textureId = importVulkanTexture(handle, width, height);

        glDisable(GL_DEPTH_TEST);
        glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        glUseProgram(shaderProgram);

        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, textureId);

        int textureLocation = glGetUniformLocation(shaderProgram, "screenTexture");
        glUniform1i(textureLocation, 0);

        glBindVertexArray(quadVAO);
        glDrawArrays(GL_TRIANGLES, 0, 6);
        glBindVertexArray(0);

        glBindTexture(GL_TEXTURE_2D, 0);

        currentTextureId = textureId;
    }
    
    private int importVulkanTexture(long handle, int width, int height) {
        if (currentTextureId != 0) {
            glDeleteTextures(currentTextureId);
        }
        
        int texture = glGenTextures();
        glBindTexture(GL_TEXTURE_2D, texture);
        
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer memoryObject = stack.mallocInt(1);
            glCreateMemoryObjectsEXT(memoryObject);
            int memObj = memoryObject.get(0);
            
            glImportMemoryWin32HandleEXT(
                memObj, 
                (long) width * height * 4,
                GL_HANDLE_TYPE_OPAQUE_WIN32_EXT, 
                handle
            );
            
            glTexStorageMem2DEXT(
                GL_TEXTURE_2D,
                1,
                GL_RGBA8,
                width,
                height,
                memObj,
                0
            );

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        }
        
        glBindTexture(GL_TEXTURE_2D, 0);
        return texture;
    }
    
    private void initShaders() {
        String vertexShaderSource = "#version 330 core\n" +
            "layout (location = 0) in vec2 aPos;\n" +
            "layout (location = 1) in vec2 aTexCoord;\n" +
            "out vec2 TexCoord;\n" +
            "void main() {\n" +
            "    gl_Position = vec4(aPos, 0.0, 1.0);\n" +
            "    TexCoord = aTexCoord;\n" +
            "}";
        
        String fragmentShaderSource = "#version 330 core\n" +
            "out vec4 FragColor;\n" +
            "in vec2 TexCoord;\n" +
            "uniform sampler2D screenTexture;\n" +
            "void main() {\n" +
            "    FragColor = texture(screenTexture, TexCoord);\n" +
            "}";
        int vertexShader = compileShader(GL_VERTEX_SHADER, vertexShaderSource);
        int fragmentShader = compileShader(GL_FRAGMENT_SHADER, fragmentShaderSource);

        shaderProgram = glCreateProgram();
        glAttachShader(shaderProgram, vertexShader);
        glAttachShader(shaderProgram, fragmentShader);
        glLinkProgram(shaderProgram);

        checkShaderProgramLinkStatus(shaderProgram);
        
        glDeleteShader(vertexShader);
        glDeleteShader(fragmentShader);
    }
    
    private int compileShader(int type, String source) {
        int shader = glCreateShader(type);
        glShaderSource(shader, source);
        glCompileShader(shader);
        checkShaderCompileStatus(shader, type);
        return shader;
    }
    
    private void checkShaderCompileStatus(int shader, int type) {
        int success = glGetShaderi(shader, GL_COMPILE_STATUS);
        if (success == GL_FALSE) {
            String infoLog = glGetShaderInfoLog(shader);
            String shaderType = (type == GL_VERTEX_SHADER) ? "VERTEX" : "FRAGMENT";
            throw new RuntimeException("Nexus OpenGL " + shaderType + " shader compilation failed: " + infoLog);
        }
    }
    
    private void checkShaderProgramLinkStatus(int program) {
        int success = glGetProgrami(program, GL_LINK_STATUS);
        if (success == GL_FALSE) {
            String infoLog = glGetProgramInfoLog(program);
            throw new RuntimeException("Nexus OpenGL shader program linking failed: " + infoLog);
        }
    }
    
    private void initQuad() {
        float[] quadVertices = {
            -1.0f,  1.0f, 0.0f, 1.0f,
            -1.0f, -1.0f, 0.0f, 0.0f,
             1.0f, -1.0f, 1.0f, 0.0f, 
            
            -1.0f,  1.0f, 0.0f, 1.0f,
             1.0f, -1.0f, 1.0f, 0.0f,
             1.0f,  1.0f, 1.0f, 1.0f
        };
        quadVAO = glGenVertexArrays();
        quadVBO = glGenBuffers();
        
        glBindVertexArray(quadVAO);
        glBindBuffer(GL_ARRAY_BUFFER, quadVBO);
        
        FloatBuffer vertexBuffer = MemoryUtil.memAllocFloat(quadVertices.length);
        vertexBuffer.put(quadVertices).flip();
        glBufferData(GL_ARRAY_BUFFER, vertexBuffer, GL_STATIC_DRAW);
        MemoryUtil.memFree(vertexBuffer);

        glVertexAttribPointer(0, 2, GL_FLOAT, false, 4 * Float.BYTES, 0L);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 2, GL_FLOAT, false, 4 * Float.BYTES, 2L * Float.BYTES);
        glEnableVertexAttribArray(1);
        
        glBindVertexArray(0);
    }
    
    public void cleanup() {
        if (currentTextureId != 0) {
            glDeleteTextures(currentTextureId);
            currentTextureId = 0;
        }
        
        if (shaderProgram != 0) {
            glDeleteProgram(shaderProgram);
            shaderProgram = 0;
        }
        
        if (quadVAO != 0) {
            glDeleteVertexArrays(quadVAO);
            quadVAO = 0;
        }
        
        if (quadVBO != 0) {
            glDeleteBuffers(quadVBO);
            quadVBO = 0;
        }
        
        initialized = false;
    }
}
