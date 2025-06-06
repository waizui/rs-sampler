//! Depth of field technique
use rs_sampler::{cam, haltonsampler};

type Real = f32;
type Rgb = image::Rgb<Real>;

fn gen_spheres() -> Vec<([Real; 3], Real)> {
    let mut vec: Vec<([Real; 3], Real)> = Vec::new();

    let nsphere = 3;

    for i in 0..nsphere {
        let ext = i as Real;
        let c = [ext, 0., -ext * 2.];
        let r = 0.7;
        vec.push((c, r));
    }

    vec.push(([3., 1., -4.], 0.1));

    vec
}

fn main() {
    use del_geo_core::vec3;
    use image::Pixel;
    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefMutIterator;
    use rayon::iter::ParallelIterator;

    let mut w = 512;
    let mut h = 512;

    let debug = false;

    if debug {
        w = 2;
        h = 2;
    }

    let mut nsamples = 32;
    if debug {
        nsamples = 2;
    }

    let lens_rad = 0.08;
    let focal_dis = 3.86;
    let fov = 60.0;

    // let campos = [1.5, 1.5, 2.5];
    // let view = [0., -1., -2.];

    let campos = [1., 0., 2.];
    let view = [0., 0., -1.];
    let mut v2w = cam::matrix_v2w(&view).1;
    // concat translation
    v2w[0 + 3 * 4] = campos[0];
    v2w[1 + 3 * 4] = campos[1];
    v2w[2 + 3 * 4] = campos[2];

    let spheres = gen_spheres();

    let mut img = vec![*Rgb::from_slice(&[0.; 3]); w * h];
    let iter = |i_pix: usize, pix: &mut Rgb| {
        let iw = i_pix % w;
        let ih = i_pix / w;

        let mut result = [0.; 3];
        for sample in 0..nsamples {
            let lensx: Real = haltonsampler::radical_inverse(sample, 0);
            let lensy: Real = haltonsampler::radical_inverse(sample, 1);

            let (ray_org, ray_dir) = cam::gen_ray_lens(
                (lensx, lensy),
                (lens_rad, focal_dis),
                (iw, ih),
                (0., 0.),
                (w, h),
                fov,
                &v2w,
            );

            if debug {
                dbg!(iw, ih);
                dbg!(ray_org);
                dbg!(ray_dir);
            }

            let mut neart = Real::INFINITY;
            let mut cntr = [0.; 3];
            for sphere in &spheres {
                let hit_res = del_geo_nalgebra::sphere::intersection_ray(
                    &nalgebra::Vector3::<f32>::from(sphere.0),
                    sphere.1,
                    &nalgebra::Vector3::<f32>::from(ray_org),
                    &nalgebra::Vector3::<f32>::from(ray_dir),
                );

                if let Some(t) = hit_res {
                    if t < neart {
                        neart = t;
                        cntr = sphere.0;
                    }
                }
            }

            if neart < Real::INFINITY {
                let hit_pos = vec3::axpy::<f32>(neart, &ray_dir, &ray_org);
                let hit_nrm = vec3::normalize(&vec3::sub(&hit_pos, &cntr));
                result = vec3::add(&result, &hit_nrm);
            }
        }

        result = vec3::scale(&result, 1. / nsamples as Real);
        pix.0 = result;
    };

    img.par_iter_mut()
        .enumerate()
        .for_each(|(i_pix, pix)| iter(i_pix, pix));

    use image::codecs::hdr::HdrEncoder;
    let file_ms = std::fs::File::create("target/dof.hdr").unwrap();
    let enc = HdrEncoder::new(file_ms);
    let _ = enc.encode(&img, w, h);
}
