package io.github.liyze09.nexus.model.block;

import io.github.liyze09.nexus.utils.VkPos;
import net.minecraft.world.phys.AABB;

import java.util.ArrayList;
import java.util.Optional;

public class CubeModel implements Model {
    public static final CubeModel INSTANCE = new CubeModel();

    private CubeModel() {}

    @Override
    public Optional<Mesh> getMesh(VisibleFaces visibleFaces, VkPos blockPos) {
        // Generate cube mesh for visible faces only
        ArrayList<Float> verticesList = new ArrayList<>();
        ArrayList<Integer> indicesList = new ArrayList<>();

        float x = blockPos.getX();
        float y = blockPos.getY();
        float z = blockPos.getZ();

        // Cube vertices in local coordinates (0-1)
        float[][] localVertices = {
            {0, 0, 0},
            {1, 0, 0},
            {1, 1, 0},
            {0, 1, 0},
            {0, 0, 1},
            {1, 0, 1},
            {1, 1, 1},
            {0, 1, 1}
        };

        // Faces: north, south, west, east, up, down
        // Each face is defined by 4 vertices (quad) and 2 triangles
        int[][] faceQuads = {
            {3, 2, 6, 7}, // north (z=1)
            {0, 1, 5, 4}, // south (z=0)
            {0, 3, 7, 4}, // west (x=0)
            {1, 2, 6, 5}, // east (x=1)
            {2, 3, 7, 6}, // up (y=1)
            {0, 1, 5, 4}  // down (y=0) - same as south but different normal
        };

        boolean[] faceVisible = {
            visibleFaces.north,
            visibleFaces.south,
            visibleFaces.west,
            visibleFaces.east,
            visibleFaces.up,
            visibleFaces.down
        };

        int vertexOffset = 0;

        for (int face = 0; face < 6; face++) {
            if (!faceVisible[face]) {
                continue;
            }
            // Add 4 vertices for this face
            for (int i = 0; i < 4; i++) {
                int vi = faceQuads[face][i];
                float vx = localVertices[vi][0] + x;
                float vy = localVertices[vi][1] + y;
                float vz = localVertices[vi][2] + z;
                verticesList.add(vx);
                verticesList.add(vy);
                verticesList.add(vz);
            }
            // Add 2 triangles (6 indices) for this quad
            indicesList.add(vertexOffset);
            indicesList.add(vertexOffset + 1);
            indicesList.add(vertexOffset + 2);
            indicesList.add(vertexOffset);
            indicesList.add(vertexOffset + 2);
            indicesList.add(vertexOffset + 3);
            vertexOffset += 4;
        }

        if (verticesList.isEmpty()) {
            return Optional.empty();
        }

        // Convert lists to arrays
        float[] vertices = new float[verticesList.size()];
        for (int i = 0; i < verticesList.size(); i++) {
            vertices[i] = verticesList.get(i);
        }
        int[] indices = new int[indicesList.size()];
        for (int i = 0; i < indicesList.size(); i++) {
            indices[i] = indicesList.get(i);
        }

        return Optional.of(new Mesh(vertices, indices));
    }

    @Override
    public ArrayList<AABB> getAABB(VisibleFaces faces, VkPos pos, float depth) {
        // For compatibility, return empty list as we don't need AABB for triangles
        return new ArrayList<>();
    }
}