#version 330 core

layout (location = 0) in vec3  inPos;
layout (location = 1) in vec3  inNormal;
layout (location = 2) in float inHeight;
layout (location = 3) in vec2  inTexCoords;
layout (location = 4) in vec3  inTangent;
layout (location = 5) in vec3  inBitangent;

out VS_OUT {
    vec3 fragPos;
    vec2 texCoords;
    vec3 tangentLightPos;
    vec3 tangentViewPos;
    vec3 tangentFragPos;
} vs_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform vec3 lightPos;
uniform vec3 viewPos;

void main() {
    vs_out.fragPos = vec3(model * vec4(inPos, 1.0)); // TODO: Add normal * height;
    vs_out.texCoords = inTexCoords;

    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vec3 adaptedTangent = normalize(normalMatrix * inTangent);
    vec3 adaptedNormal  = normalize(normalMatrix * inNormal);
    adaptedTangent = normalize(adaptedTangent - dot(adaptedTangent, adaptedNormal) *  adaptedNormal);
    vec3 adaptedBitangent = cross(adaptedTangent, adaptedNormal);

    mat3 TBN = transpose(mat3(adaptedTangent, adaptedBitangent, adaptedNormal));
    vs_out.tangentLightPos = TBN * lightPos;
    vs_out.tangentViewPos = TBN * viewPos;
    vs_out.tangentFragPos = TBN * vs_out.fragPos;

    gl_Position = projection * view * model * vec4(inPos, 1.0);
}
