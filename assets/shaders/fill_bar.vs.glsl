#version 330

in vec3 vertexPosition;
in vec3 vertexNormal;
in vec2 vertexTexCoord;

out vec2 uv;
out vec2 size;

uniform mat4 mvp;
uniform mat4 matView;
uniform mat4 matProjection;
uniform mat4 matModel;
uniform mat4 matNormal;

void main()
{
    mat4 matModelView = matView * matModel;

    matModelView[0][0] = 1.0;
    matModelView[0][1] = 0.0;
    matModelView[0][2] = 0.0;

    matModelView[1][0] = 0.0;
    matModelView[1][1] = 1.0;
    matModelView[1][2] = 0.0;

    matModelView[2][0] = 0.0;
    matModelView[2][1] = 0.0;
    matModelView[2][2] = 1.0;

    vec4 scale = vec4(matModel[0][0], matModel[1][1], matModel[2][2], 1.0);
    vec4 vertexPosition = vec4(vertexPosition, 1.0) * scale;

    uv = vertexTexCoord;
    size = scale.xy;
    gl_Position = matProjection * matModelView * vertexPosition;
}
