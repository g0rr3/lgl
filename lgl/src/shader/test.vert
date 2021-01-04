#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

out vec3 frag_ourColor;
out vec2 frag_TexCoord;

void main()
{
    gl_Position = vec4(aPos, 1.0);
    frag_ourColor = aColor;
    frag_TexCoord = aTexCoord;
}