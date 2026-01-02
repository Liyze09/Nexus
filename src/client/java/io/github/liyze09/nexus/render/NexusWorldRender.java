package io.github.liyze09.nexus.render;

import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.RenderTarget;
import com.mojang.blaze3d.resource.GraphicsResourceAllocator;
import io.github.liyze09.nexus.NexusClientMain;
import io.github.liyze09.nexus.chunk.NexusChunkBuilder;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import net.minecraft.client.Camera;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.CloudRenderer;
import net.minecraft.client.renderer.LevelRenderer;
import net.minecraft.client.renderer.RenderBuffers;
import net.minecraft.client.renderer.SectionOcclusionGraph;
import net.minecraft.client.renderer.blockentity.BlockEntityRenderDispatcher;
import net.minecraft.client.renderer.chunk.SectionRenderDispatcher;
import net.minecraft.client.renderer.culling.Frustum;
import net.minecraft.client.renderer.entity.EntityRenderDispatcher;
import net.minecraft.client.renderer.feature.FeatureRenderDispatcher;
import net.minecraft.client.renderer.state.LevelRenderState;
import net.minecraft.core.BlockPos;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.state.BlockState;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.joml.Matrix4f;
import org.joml.Vector4f;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class NexusWorldRender extends LevelRenderer {
    private static final Logger LOGGER = LoggerFactory.getLogger(NexusWorldRender.class);
    @Nullable
    private volatile ClientLevel world = null;
    @SuppressWarnings("unused")
    private Minecraft minecraft;
    private ExternalImageRender renderer;
    private volatile boolean isCompleted = true;
    private final long nativeContext;
    private int width;
    private int height;
    public volatile NexusChunkBuilder builder = null;
    private volatile boolean isClosed = false;
    public NexusWorldRender(Minecraft minecraft, EntityRenderDispatcher entityRenderDispatcher, BlockEntityRenderDispatcher blockEntityRenderDispatcher, RenderBuffers renderBuffers, LevelRenderState levelRenderState, FeatureRenderDispatcher featureRenderDispatcher) {
        super(minecraft, entityRenderDispatcher, blockEntityRenderDispatcher, renderBuffers, levelRenderState, featureRenderDispatcher);
        this.nativeContext = NexusClientMain.initNative();
        this.minecraft = minecraft;
        this.width = minecraft.getWindow().getWidth();
        this.height = minecraft.getWindow().getHeight();
        NexusClientMain.resize(nativeContext, width, height);
        long r = NexusClientMain.getGLReady(nativeContext);
        long c = NexusClientMain.getGLComplete(nativeContext);
        this.renderer = new ExternalImageRender(r, c);
        LOGGER.info("NexusWorldRender created.");
    }

    @Override
    public void close() {
        checkIfClosed();
        super.close();
        builder.close();
        isClosed = true;
        renderer.close();
        NexusClientMain.close(nativeContext);
    }

    public boolean isClosed() {
        return isClosed;
    }
    @Nullable
    public ClientLevel getWorld() {
        return world;
    }

    private void checkIfClosed() {
        if (isClosed) {
            throw new IllegalStateException("Renderer has already closed.");
        }
    }

    @Override
    public void onResourceManagerReload(ResourceManager resourceManager) {}

    @Override
    public void initOutline() {}

    @Override
    public void doEntityOutline() {}

    @Override
    protected boolean shouldShowEntityOutlines() {
        return true;
    }

    @Override
    public void setLevel(@Nullable ClientLevel clientLevel) {
        this.world = clientLevel;
        if (clientLevel != null) {
            this.builder = new NexusChunkBuilder(clientLevel);
        } else {
            this.builder = null;
        }

    }

    @Override
    public void allChanged() {}

    @Override
    public void resize(int i, int j) {
        this.width = i;
        this.height = j;
        NexusClientMain.resize(nativeContext, width, height);
    }

    @Override
    public @Nullable String getSectionStatistics() {
        return null;
    }

    @Override
    public @Nullable SectionRenderDispatcher getSectionRenderDispatcher() {
        return null;
    }

    @Override
    public double getTotalSections() {
        return 0.0;
    }

    @Override
    public double getLastViewDistance() {
        return -1D;
    }

    @Override
    public int countRenderedSections() {
        return 12;
    }

    @Override
    public @Nullable String getEntityStatistics() {
        return null;
    }

    @Override
    public void addRecentlyCompiledSection(SectionRenderDispatcher.RenderSection _renderSection) {

    }

    

    @Override
    public void renderLevel(GraphicsResourceAllocator graphicsResourceAllocator, DeltaTracker deltaTracker, boolean bl, Camera camera, Matrix4f matrix4f, Matrix4f matrix4f2, Matrix4f matrix4f3, GpuBufferSlice gpuBufferSlice, Vector4f vector4f, boolean bl2) {
        checkIfClosed();
        this.isCompleted = false;
        long t = NexusClientMain.refresh(nativeContext);
        renderer.refresh(t, width, height);
        renderer.render();
        this.isCompleted = true;
    }

    @Override
    public void endFrame() {}

    @Override
    public void captureFrustum() {}

    @Override
    public void killFrustum() {
        super.killFrustum();
    }

    @Override
    public void tick(Camera camera) {

    }

    @Override
    public void blockChanged(BlockGetter blockGetter, BlockPos blockPos, BlockState blockState, BlockState blockState2, int i) {

    }

    @Override
    public void setBlocksDirty(int i, int j, int k, int l, int m, int n) {

    }

    @Override
    public void setBlockDirty(BlockPos blockPos, BlockState blockState, BlockState blockState2) {

    }

    @Override
    public void setSectionDirtyWithNeighbors(int i, int j, int k) {

    }

    @Override
    public void setSectionRangeDirty(int i, int j, int k, int l, int m, int n) {

    }

    @Override
    public void setSectionDirty(int i, int j, int k) {

    }

    @Override
    public void onSectionBecomingNonEmpty(long l) {

    }

    @Override
    public void destroyBlockProgress(int i, BlockPos blockPos, int j) {
        super.destroyBlockProgress(i, blockPos, j);
    }

    @Override
    public boolean hasRenderedAllSections() {
        return isCompleted;
    }

    @Override
    public void onChunkReadyToRender(ChunkPos chunkPos) {

    }

    @Override
    public void needsUpdate() {

    }

    @Override
    public @Nullable RenderTarget entityOutlineTarget() {
        return null;
    }

    @Override
    public @Nullable RenderTarget getTranslucentTarget() {
        return null;
    }

    @Override
    public @Nullable RenderTarget getItemEntityTarget() {
        return null;
    }

    @Override
    public @Nullable RenderTarget getParticlesTarget() {
        return null;
    }

    @Override
    public @Nullable RenderTarget getWeatherTarget() {
        return null;
    }

    @Override
    public @Nullable RenderTarget getCloudsTarget() {
        return null;
    }

    @Override
    public @NotNull ObjectArrayList<SectionRenderDispatcher.RenderSection> getVisibleSections() {
        return super.getVisibleSections();
    }

    @Override
    public @NotNull SectionOcclusionGraph getSectionOcclusionGraph() {
        return super.getSectionOcclusionGraph();
    }

    @Override
    public @Nullable Frustum getCapturedFrustum() {
        return super.getCapturedFrustum();
    }

    @Override
    public @NotNull CloudRenderer getCloudRenderer() {
        return super.getCloudRenderer();
    }
}
