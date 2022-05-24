#version 330 core

out vec4 Color;

in VS_OUT {
    vec3 fragPos;
    vec3 fragNormal;
    vec2 texCoords;
    vec3 tangentLightPos;
    vec3 tangentViewPos;
    vec3 tangentFragPos;
} fs_in;

uniform sampler2D baseMap;
uniform sampler2D normalMap;
uniform sampler2D heightMap;

uniform vec3 lightPos;
uniform vec3 viewPos;

void main() {
    vec3 normal = texture(normalMap, fs_in.texCoords).rgb;
    normal = normalize(normal * 2.0 - 1.0);

    // Base color
    vec3 color = texture(baseMap, fs_in.texCoords).rgb;

    // Ambient light
    vec3 ambient = 0.1 * color;

    // Diffuse light
    vec3 lightDir = normalize(fs_in.tangentLightPos - fs_in.tangentFragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * color;

    // Specular light
    vec3 viewDir = normalize(fs_in.tangentViewPos - fs_in.tangentFragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 32.0);
    vec3 specular = vec3(0.2) * spec;

    Color = vec4(ambient + diffuse + specular, 1.0);
}