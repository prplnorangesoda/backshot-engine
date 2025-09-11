#version 430 core

layout (location = 0) in vec3 attribute_Position;
layout (location = 1) in vec3 attribute_Colour; 


out vec3 vertexColour;

void main()
{
    gl_Position = vec4(attribute_Position.x, attribute_Position.y, attribute_Position.z, 1.0);
    vertexColour = attribute_Colour;
}