#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}


struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 0.0, 1.0),
    );
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}



@group(2) @binding(0) var material_sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var material_sdf_sampler: sampler;

fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}


fn contour(d: f32, w: f32) -> f32 {
    return smoothstep(0.5 - w, 0.5 + w, d);
}

fn samp(uv: vec2<f32>, w: f32) -> f32 {
    let sample = textureSample(material_sdf_texture, material_sdf_sampler, uv);
    // let dist = median(sample.r, sample.g, sample.b);
    let dist = sample.a;
    return contour(dist, w);
}

fn safeNormalize(v: vec2<f32>) -> vec2<f32> {
    let len = length(v);
    if len > 0 {
        return v / len;
    } else {
        return vec2(0.0, 0.0);
    }
}

const kThickness : f32 = 0.125;
const kNormalization : f32 = kThickness * 0.5 * sqrt(2.0);

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // let sigDist = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv).r;
    // let w = fwidth(sigDist);
    // let opacity = smoothstep(0.5 - w, 0.5 + w, sigDist);
    // return vec4(1, 1, 1, opacity);

    // adapted from: https://jvm-gaming.org/t/solved-signed-distance-field-fonts-look-crappy-at-small-pt-sizes/49617/7
    let sample = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv);
    // let dist = median(sample.r, sample.g, sample.b);
    let dist = sample.a;
    let width = fwidth(dist);
    var alpha = contour(dist, width) ;
    // let dscale = 0.354; // 0.354; // half of 1/sqrt2; you can play with this
    // let duv = dscale * (dpdx(mesh.uv) + dpdy(mesh.uv));
    // let box = vec4(mesh.uv - duv, mesh.uv + duv);
    // let asum = samp(box.xy, width) + samp(box.zw, width) + samp(box.xw, width) + samp(box.zy, width);
    // alpha = (alpha + 0.5 * asum) / 3.0;
    return vec4(mesh.color.rgb, alpha);

    // adapted from Cinder: https://github.com/paulhoux/Cinder-SDFText/blob/565b24e0d886ac6b8dbccdeed0d9a9d4bec3d45b/src/cinder/gl/SdfText.cpp
    // let texSize = textureDimensions(material_sdf_texture, 0);
	// let uv = vec2(mesh.uv.x * f32(texSize.x),  mesh.uv.y * f32(texSize.y));
	// let jdx = dpdx(uv);
	// let jdy = dpdy(uv);
	// // Sample SDF texture (3 channels).
	// let sample = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv).a;
	// // Calculate signed distance (in texels).
	// // float sigDist = median( sample.r, sample.g, sample.b ) - 0.5;
    // let sigDist = sample - 0.5;
	// // For proper anti-aliasing, we need to calculate signed distance in pixels. We do this using derivatives.
	// let gradDist = safeNormalize(vec2(dpdx(sigDist), dpdy(sigDist)));
	// let grad = vec2( gradDist.x * jdx.x + gradDist.y * jdy.x, gradDist.x * jdx.y + gradDist.y * jdy.y );
	
	// let afwidth = min(kNormalization * length(grad), 0.5);
	// let opacity = smoothstep(0.0 - afwidth, 0.0 + afwidth, sigDist);
    // // If enabled apply pre-multiplied alpha. Always apply gamma correction.
    // return vec4(mesh.color.rgb, opacity);


    // old
    // let msd = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv);
    // let sd = median(msd.r, msd.g, msd.b) - 0.5;
    // let sd = msd.a - 0.5;

    // TODO: cache this
    // let texSize = textureDimensions(material_sdf_texture, 0);
    // let unitRange : vec2<f32> = vec2(6.0 / f32(texSize.x), 6.0 / f32(texSize.y));

    // let screenTexSize = vec2(1.0) / fwidth(mesh.uv);
    // let screenPxRange = max(0.5 * dot(unitRange, screenTexSize), 1.0);

    // let screenPxDistance = screenPxRange * sd;
    // let opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
    // return vec4(mesh.color.rgb, opacity);

    //let sdf = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv);
    //let inside : f32 = f32 (sdf.x);
    //return vec4(sdf.xyz, 1.0);
}