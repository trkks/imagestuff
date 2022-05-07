## imagestuff

Collection of small command-line programs that have something to do with image-files.

### ascii
Turn an image into ASCII-text.

Running this for the [Rust](https://www.rust-lang.org/) logo with: 
```
cargo run --bin ascii .\images\rustlogo.png -w 20 --inverted --palette "░▒▓█"
```
produces:
```
▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
▒▒▒▒▒▒▒▒▒▒▒▒████████████████▒▒▒▒▒▒▒▒▒▒▒▒
▒▒▒▒▒▒▒▒▓▓████████▓▓▓▓████████▓▓▒▒▒▒▒▒▒▒
▒▒▒▒▒▒██████▓▓▒▒▒▒▓▓▓▓▒▒▒▒▓▓██████▒▒▒▒▒▒
▒▒▒▒▓▓████▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒████▓▓▒▒▒▒
▒▒▒▒██████████████████████▓▓▒▒▒▒████▒▒▒▒
▒▒██████████████████████████▓▓▒▒██████▒▒
▒▒██▓▓▓▓▓▓██████▓▓▒▒▒▒▓▓██████▒▒▓▓▓▓██▒▒
▓▓████▓▓▒▒██████▓▓▒▒▓▓▓▓████▓▓▒▒▓▓████▓▓
▓▓██▓▓▒▒▒▒██████████████████▒▒▒▒▒▒████▓▓
▓▓██▓▓▒▒▒▒████████▓▓████████▓▓▒▒▒▒████▓▓
▓▓████▒▒▒▒██████▓▓▒▒▒▒▓▓████▓▓▒▒▓▓████▓▓
▒▒██████████████████▒▒▓▓██████████████▒▒
▒▒██████████████████▒▒▒▒██████████████▒▒
▒▒▒▒████▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓████▒▒▒▒
▒▒▒▒▓▓████▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓████▓▓▒▒▒▒
▒▒▒▒▒▒████▓▓██▒▒▒▒▒▒▒▒▒▒▒▒██▓▓████▒▒▒▒▒▒
▒▒▒▒▒▒▒▒▓▓████████████████████▓▓▒▒▒▒▒▒▒▒
▒▒▒▒▒▒▒▒▒▒▒▒████████████████▒▒▒▒▒▒▒▒▒▒▒▒
▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
```

### raycast
Make an image of a scene described in JSON with 3D-primitives. Based on 
various ray-tracing guides and some code initially translated from a university course assignment.

Running:
```
cargo run --bin raycast .\raycast_scenes\plane_sphere_triangle.json -w 1024 -h 512
```
produces:
![](https://github.com/trkks/imagestuff/blob/main/renders/plane_sphere_triangle_1024x512.png)

### atlas
Glue a list of images together into a row. Can be used to make batches of animation-sheets of game-sprites for example.

### lerp 
"Linear interpolation" between images. Produces a third new image
that is faded halfway between two. Originally this
started out as an automation tool for making fades between animation frames,
but could now be better suited for __nothing__.
