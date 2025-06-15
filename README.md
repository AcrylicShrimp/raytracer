# raytracer

This is a simple raytracer written in Rust.

![result-1](./docs/result-1.png)

## implemented BRDFs

- Lambertian
- [Disney](https://media.disneyanimation.com/uploads/production/publication_asset/48/asset/s2012_pbs_disney_brdf_notes_v3.pdf)

## to-dos

- [x] fix clearcoat
- [ ] apply specular tint
- [x] implement NEE and MIT
- [ ] implement subsurface
- [ ] implement sheer
- [ ] implement anisotropic

## roadmap

1. introduce mesh type (that can be loaded from model files)
2. index scene in BVH-like structure for fast and efficient ray-triangle intersection
3. use `wgpu` for gpu-acceleration
