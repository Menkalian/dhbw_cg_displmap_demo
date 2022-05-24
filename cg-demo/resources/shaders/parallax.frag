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

const float heightScale = 0.1;

vec2 ParallaxMapping(vec2 texCoords, vec3 viewDir)
{
    float height =  texture(heightMap, texCoords).r;
    return texCoords - viewDir.xy * (height * heightScale);
}

void main() {
    // offset texture coordinates with Parallax Mapping
    vec3 viewDir = normalize(fs_in.tangentViewPos - fs_in.tangentFragPos);
    vec2 texCoords = fs_in.texCoords;

    texCoords = ParallaxMapping(fs_in.texCoords, viewDir);
    if (texCoords.x > 1.0 || texCoords.y > 1.0 || texCoords.x < 0.0 || texCoords.y < 0.0)
    discard;

    vec3 normal = texture(normalMap, texCoords).rgb;
    normal = normalize(normal * 2.0 - 1.0);

    // Base color
    vec3 color = texture(baseMap, texCoords).rgb;

    // Ambient light
    vec3 ambient = 0.1 * color;

    // Diffuse light
    vec3 lightDir = normalize(fs_in.tangentLightPos - fs_in.tangentFragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * color;

    // Specular light
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 32.0);
    vec3 specular = vec3(0.2) * spec;

    Color = vec4(ambient + diffuse + specular, 1.0);
}