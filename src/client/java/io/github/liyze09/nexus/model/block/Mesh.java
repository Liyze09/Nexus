package io.github.liyze09.nexus.model.block;

public class Mesh {
    public float[] vertices;
    public int[] indices;

    public Mesh(float[] vertices, int[] indices) {
        this.vertices = vertices;
        this.indices = indices;
    }
}   