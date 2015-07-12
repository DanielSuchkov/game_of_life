#version 140

in vec3 position;
in vec3 normal;
in vec4 row0;
in vec4 row1;
in vec4 row2;
in vec4 row3;
out vec3 v_position;
out vec3 v_normal;
out vec3 v_color;
uniform mat4 mvp;

void main() {
    mat4 model = mat4(row0, row1, row2, row3);
    v_position = position;
    v_normal = mat3(model) * normal;
    v_color = vec3(float(gl_InstanceID) / (24.0*24*24), 0.5 + float(gl_InstanceID - (24.0*24*12)) / (24.0*24*24), 1.0 - float(gl_InstanceID) / (24.0*24*24));
    gl_Position = mvp * model * vec4(position, 1.0);
}
