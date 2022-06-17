#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 offset;

layout(location = 0) out vec4 f_colour;

void main() {
	gl_Position = vec4(position + offset, 0.0, 1.0);
    f_colour = vec4((offset + position + 1.0) / 2.0, 1.0, 1.0);
}