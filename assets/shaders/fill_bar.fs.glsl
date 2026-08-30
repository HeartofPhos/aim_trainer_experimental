#version 330

in vec2 size;
in vec2 uv;

out vec4 finalColor;

uniform float borderRadius;
uniform float roundingFactor;
uniform vec4 fillColor;
uniform vec4 borderColor;
uniform vec4 emptyColor;
uniform float fill;
uniform int fillAxis;
uniform int fillFlip;

float sd_rounded_box(vec2 p, vec2 b, vec4 r) {
    float x = r.x;
    float y = r.y;
    x = mix(r.z, r.x, float(p.x > 0.0));
    y = mix(r.w, r.y, float(p.x > 0.0));
    x = mix(y, x, float(p.y > 0.0));
    vec2 q = abs(p) - b + x;
    return min(max(q.x, q.y), 0.) + length(max(q, vec2(0.))) - x;
}

vec2 op_translate(vec2 p, vec2 t) {
    return p - t;
}

float min_element(vec2 p) {
    return min(p.x, p.y);
}

void main()
{
    vec2 uv = mix(uv, (1.0 - uv), fillFlip);
    vec2 sample = uv * size;

    vec2 extents = size * 0.5;
    vec2 box_extents = extents - borderRadius;
    vec4 box_round = vec4(min_element(box_extents) * roundingFactor);

    float bar_dist = sd_rounded_box(op_translate(sample, extents), box_extents, box_round);
    bool bar_mask = bar_dist < 0;

    float min = extents[fillAxis] - box_extents[fillAxis];
    float max = extents[fillAxis] + box_extents[fillAxis];
    float fillThreshold = mix(min, max, fill);

    vec4 bar_color = mix(emptyColor, fillColor, float(sample[fillAxis] < fillThreshold));

    float border_dist = bar_dist - borderRadius;
    bool border_mask = !bar_mask && border_dist < 0;

    finalColor = bar_color * float(bar_mask) + borderColor * float(border_mask);

    if (finalColor.a == 0) { discard; }
}
