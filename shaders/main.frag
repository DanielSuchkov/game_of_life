#version 140

in vec3 v_normal;
in vec4 v_color;
out vec4 f_color;
uniform mat4 mvp;

const vec4 LIGHT = -vec4(-0.7, -0.8, 1.0, 0.0);

void main() {
    if (v_color.a < 0.00001) discard;
    float lum = max(dot(normalize(-v_normal), normalize(mvp * LIGHT).xyz), 0.0);
    f_color = (0.1 + 0.9 * lum) * v_color;
    f_color.a = v_color.a;
}
