## imagestuff

Collection of small command-line programs that have something to do with
image-files.

### mosaic
Maps pixels of an image to text characters.

```
cargo run --bin ascii ./images/rustlogo.png -w 20 --inverted --palette "░▒▓█"
```
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
Make an image of a scene described in JSON with 3D-primitives. Based on various
ray-tracing guides and some code initially translated from a university course
assignment.

Running:
```
cargo run --bin raycast ./raycast/scenes/plane_sphere_triangle.json -w 1024 -h 512
```
produces:
![](raycast/renders/plane_sphere_triangle_1024x512.png)

### atlas
Glue a list of images together into a row. Can be used to make batches of
animation-sheets of game-sprites for example.
