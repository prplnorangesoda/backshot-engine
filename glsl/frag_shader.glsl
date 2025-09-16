#version 430 core
layout(location = 0) out vec4 FragColor;

layout(location = 0) in vec3 vertexColour;

void main() { FragColor = vec4(vertexColour, 1.0f); }
