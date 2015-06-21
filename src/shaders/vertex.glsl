#version 140

in vec3 position;
in vec3 normal;
in vec4 model0;
in vec4 model1;
in vec4 model2;
in vec4 model3;
out vec3 v_position;
out vec3 v_normal;
out vec3 v_color;
uniform mat4 mvp;

void main() {
    mat4 model = mat4(model0, model1, model2, model3);
    v_position = position;
    v_normal = mat3(model) * normal;
    v_color = vec3(float(gl_InstanceID) / 100.0, 0.5 + float(gl_InstanceID - 50) / 200.0, 1.0 - float(gl_InstanceID) / 100.0);
    gl_Position = mvp * model * vec4(position, 1.0);
}
