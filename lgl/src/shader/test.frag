#version 330 core
out vec4 FragColor;
  
in vec3 frag_ourColor;
in vec2 frag_TexCoord;

uniform sampler2D ourTexture;

void main()
{
    FragColor = texture(ourTexture, frag_TexCoord) * vec4(frag_ourColor, 1.0);
}