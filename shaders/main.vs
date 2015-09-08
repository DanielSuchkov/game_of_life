#version 400

in vec3 position;
in vec3 normal;
in vec3 pos;
in vec4 color;
in float scale_factor;

out vec3 v_position;
out vec3 v_normal;
out vec4 v_color;
uniform mat4 mvp;

void main() {
    v_position = position;
    v_normal = normal;
    v_color = color;
    gl_Position = mvp * vec4(position * scale_factor + pos, 1.0);
}
