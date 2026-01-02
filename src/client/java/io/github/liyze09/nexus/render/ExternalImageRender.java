package io.github.liyze09.nexus.render;
import org.lwjgl.system.MemoryStack;
import static org.lwjgl.opengl.GL30C.*;
import java.io.Closeable;
import static org.lwjgl.opengl.EXTMemoryObject.*;
import static org.lwjgl.opengl.EXTMemoryObjectWin32.*;
import static org.lwjgl.opengl.EXTSemaphore.*;
import static org.lwjgl.opengl.EXTSemaphoreWin32.*;
import static org.lwjgl.opengl.EXTSemaphoreWin32.GL_HANDLE_TYPE_OPAQUE_WIN32_EXT;
import static org.lwjgl.opengl.GL11C.GL_COLOR_BUFFER_BIT;
import static org.lwjgl.opengl.GL11C.GL_DEPTH_BUFFER_BIT;
import static org.lwjgl.opengl.GL11C.GL_FALSE;
import static org.lwjgl.opengl.GL11C.GL_LINEAR;
import static org.lwjgl.opengl.GL11C.GL_RGBA8;
import static org.lwjgl.opengl.GL11C.GL_TEXTURE_2D;
import static org.lwjgl.opengl.GL11C.GL_TEXTURE_MAG_FILTER;
import static org.lwjgl.opengl.GL11C.GL_TEXTURE_MIN_FILTER;
import static org.lwjgl.opengl.GL11C.GL_TEXTURE_WRAP_S;
import static org.lwjgl.opengl.GL11C.GL_TEXTURE_WRAP_T;
import static org.lwjgl.opengl.GL11C.GL_TRIANGLE_STRIP;
import static org.lwjgl.opengl.GL11C.GL_TRUE;
import static org.lwjgl.opengl.GL11C.glBindTexture;
import static org.lwjgl.opengl.GL11C.glClear;
import static org.lwjgl.opengl.GL11C.glClearColor;
import static org.lwjgl.opengl.GL11C.glDeleteTextures;
import static org.lwjgl.opengl.GL11C.glDrawArrays;
import static org.lwjgl.opengl.GL11C.glFlush;
import static org.lwjgl.opengl.GL11C.glGenTextures;
import static org.lwjgl.opengl.GL11C.glTexParameteri;
import static org.lwjgl.opengl.GL11C.glViewport;
import static org.lwjgl.opengl.GL12C.GL_CLAMP_TO_EDGE;
import static org.lwjgl.opengl.GL13C.GL_TEXTURE0;
import static org.lwjgl.opengl.GL13C.glActiveTexture;
import static org.lwjgl.opengl.GL20C.GL_COMPILE_STATUS;
import static org.lwjgl.opengl.GL20C.GL_FRAGMENT_SHADER;
import static org.lwjgl.opengl.GL20C.GL_LINK_STATUS;
import static org.lwjgl.opengl.GL20C.GL_VERTEX_SHADER;
import static org.lwjgl.opengl.GL20C.glAttachShader;
import static org.lwjgl.opengl.GL20C.glCompileShader;
import static org.lwjgl.opengl.GL20C.glCreateProgram;
import static org.lwjgl.opengl.GL20C.glCreateShader;
import static org.lwjgl.opengl.GL20C.glDeleteProgram;
import static org.lwjgl.opengl.GL20C.glDeleteShader;
import static org.lwjgl.opengl.GL20C.glGetProgramInfoLog;
import static org.lwjgl.opengl.GL20C.glGetProgrami;
import static org.lwjgl.opengl.GL20C.glGetShaderInfoLog;
import static org.lwjgl.opengl.GL20C.glGetShaderi;
import static org.lwjgl.opengl.GL20C.glGetUniformLocation;
import static org.lwjgl.opengl.GL20C.glLinkProgram;
import static org.lwjgl.opengl.GL20C.glShaderSource;
import static org.lwjgl.opengl.GL20C.glUniform1i;
import static org.lwjgl.opengl.GL20C.glUseProgram;


public class ExternalImageRender implements Closeable {
    private int program;
    private int glReadySemaphore;
    private int glCompleteSemaphore;
    private int memoryObject;
    private int texture;
    private int vao;
    private int fbo;
    private long externalMemory;
    
    private int width;
    private int height;
    
    private static final String VERTEX_SHADER_SOURCE = """
        #version 330 core

        layout(location = 0) in vec2 aPosition;
        out vec2 TexCoord;

        void main() {
            gl_Position = vec4(aPosition, 0.0, 1.0);
            TexCoord = aPosition * 0.5 + 0.5;
        }""";
    
    private static final String FRAGMENT_SHADER_SOURCE = """
        #version 330 core

        in vec2 TexCoord;
        out vec4 FragColor;

        uniform sampler2D texSampler;

        void main() {
            FragColor = texture(texSampler, TexCoord);
        }""";
    
    public ExternalImageRender(long glReady, long glComplete) {
        initialize(glReady, glComplete);
    }
    
    private void initialize(long glReady, long glComplete) {
        int vertexShader = createShader(GL_VERTEX_SHADER, VERTEX_SHADER_SOURCE);
        int fragmentShader = createShader(GL_FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
        
        program = glCreateProgram();
        glAttachShader(program, vertexShader);
        glAttachShader(program, fragmentShader);
        glLinkProgram(program);
        
        checkProgramLinkStatus(program);
        
        glDeleteShader(vertexShader);
        glDeleteShader(fragmentShader);
        
        int[] semaphores = new int[2];
        glGenSemaphoresEXT(semaphores);
        glReadySemaphore = semaphores[0];
        glCompleteSemaphore = semaphores[1];
        
        int handleType = getPlatformHandleType();
        glImportSemaphoreWin32HandleEXT(glReadySemaphore, handleType, glReady);
        glImportSemaphoreWin32HandleEXT(glCompleteSemaphore, handleType, glComplete);
        
        vao = glGenVertexArrays();
        glBindVertexArray(vao);

        fbo = glGenFramebuffers();
        glViewport(0, 0, width, height);
    }
    
    public void refresh(long externalMemory, int width, int height) {
        if (externalMemory == this.externalMemory && width == this.width && height == this.height) {
            return;
        }
        this.width = width;
        this.height = height;
        
        if (texture != 0) {
            glDeleteTextures(texture);
        }
        
        if (memoryObject != 0) {
            glDeleteMemoryObjectsEXT(memoryObject);
        }
        
        try (MemoryStack stack = MemoryStack.stackPush()) {
            int[] memoryObjects = new int[1];
            glCreateMemoryObjectsEXT(memoryObjects);
            memoryObject = memoryObjects[0];
            
            int dedicated = GL_TRUE;
            glMemoryObjectParameterivEXT(memoryObject, 
                GL_DEDICATED_MEMORY_OBJECT_EXT, 
                stack.ints(dedicated));
            
            int handleType = getPlatformHandleType();
            long allocationSize = width * height * 4L;
            glImportMemoryWin32HandleEXT(memoryObject, allocationSize, handleType, externalMemory);
            
            texture = glGenTextures();
            glBindTexture(GL_TEXTURE_2D, texture);
            
            glTexStorageMem2DEXT(GL_TEXTURE_2D, 1, GL_RGBA8, width, height, 
                memoryObject, 0);
            
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
            
            glBindTexture(GL_TEXTURE_2D, 0);
            
            glBindFramebuffer(GL_FRAMEBUFFER, fbo);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, 
                GL_TEXTURE_2D, texture, 0);
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
        }
        
        glViewport(0, 0, width, height);
    }
    
    public void render() {
        waitForVulkanSemaphore();
        
        glBindFramebuffer(GL_FRAMEBUFFER, 0);
        
        glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        glUseProgram(program);

        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture);
        glUniform1i(glGetUniformLocation(program, "texSampler"), 0);

        glBindVertexArray(vao);
        
        glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);

        glBindVertexArray(0);
        glUseProgram(0);
        glBindTexture(GL_TEXTURE_2D, 0);
        
        signalVulkanSemaphore();
        glFlush();
    }
    
    private void waitForVulkanSemaphore() {
        int srcLayout = GL_LAYOUT_COLOR_ATTACHMENT_EXT;
        int[] textures = {texture};
        int[] srcLayouts = {srcLayout};
        
        glWaitSemaphoreEXT(glReadySemaphore, emptyIntList, textures, srcLayouts);
    }
    
    private void signalVulkanSemaphore() {
        int dstLayout = GL_LAYOUT_SHADER_READ_ONLY_EXT;
        int[] textures = {texture};
        int[] dstLayouts = {dstLayout};
        
        glSignalSemaphoreEXT(glCompleteSemaphore, emptyIntList, textures, dstLayouts);
    }
    
    private int createShader(int type, String source) {
        int shader = glCreateShader(type);
        glShaderSource(shader, source);
        glCompileShader(shader);
        
        if (glGetShaderi(shader, GL_COMPILE_STATUS) == GL_FALSE) {
            glDeleteShader(shader);
            throw new RuntimeException("Shader compilation failed: " + glGetShaderInfoLog(shader));
        }
        
        return shader;
    }
    
    private void checkProgramLinkStatus(int program) {
        if (glGetProgrami(program, GL_LINK_STATUS) == GL_FALSE) {
            throw new RuntimeException("Program linking failed: " + glGetProgramInfoLog(program));
        }
    }
    
    private int getPlatformHandleType() {
        return GL_HANDLE_TYPE_OPAQUE_WIN32_EXT;
    }
    
    @Override
    public void close() {
        if (program != 0) {
            glDeleteProgram(program);
        }
        
        if (texture != 0) {
            glDeleteTextures(texture);
        }
        
        if (memoryObject != 0) {
            glDeleteMemoryObjectsEXT(memoryObject);
        }
        
        if (glReadySemaphore != 0) {
            glDeleteSemaphoresEXT(glReadySemaphore);
        }
        
        if (glCompleteSemaphore != 0) {
            glDeleteSemaphoresEXT(glCompleteSemaphore);
        }
        
        if (vao != 0) {
            glDeleteVertexArrays(vao);
        }
        
        if (fbo != 0) {
            glDeleteFramebuffers(fbo);
        }
    }
    private int[] emptyIntList = new int[0];
}