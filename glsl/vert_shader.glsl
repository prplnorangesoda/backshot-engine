#version 410 core
layout (location = 0) in vec3 attribute_Position;

void main()
{
    gl_Position = vec4(attribute_Position.x, attribute_Position.y, attribute_Position.z, 1.0);
}