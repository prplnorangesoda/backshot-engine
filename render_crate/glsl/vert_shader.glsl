#version 430 core

layout(location = 0) in vec3 attribute_Position;
layout(location = 1) in vec3 attribute_Colour;

layout(location = 0) out vec3 vertexColour;

void main() {
  gl_Position = vec4(attribute_Position.xyz, 1.0);
  vertexColour = attribute_Colour;
}