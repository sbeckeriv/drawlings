# drawlings

I was gifted a Leonardo Da Vinci's drawing machine [1] recently. It was a lot of fun to build and use. I drew all the program disks I received.
After watching it work for a while I thought about what it was doing. Simply each petal moves the arm along the X and y axis. I asked the creators
if they already had software to convert images to disks. They said that they were all hand created.

## Concept

1. trace a line image and record its points
1. turn those points in to x and y plots
1. create a disk image one for each x and y
1. find a friend with a laser cutter
1. enjoy

## Rules

1. all lines are in black rbga(0,0,0,1)
1. the line must close back on itself
1. no line crosses
   1. if a line is to cross it must end with a unique color to the direction of the crossing. 
   1. the crossed point must be white 
   1. the other side of the line must have the matching color line 
   1. the other side's color line must touch the black line again
 
# Process

### basics

every command takes a required input, output file location and a subcommand. I do not create folders for you.

`cargo run -- test.toml output  disk_images`

verbose levels can output files in different stages. If the trace is not working it will show you around which point failed. 
I need to document this better but you add more -v1's

`cargo run --release -- test_data/line.png output -v1 -v1 -v1 -v1 process_file`


### stage 1 (vector_dump)

Trace the image and dump the line points to a text (toml) file. This allows a person to edit points.

### debug (debug_image)

Takes a vector toml file and creates an image. It should be the final image that is produce by the disks. Why? If you edit the file it is a nice tool.
There might be unwanted artificts from width of the line that can show up. 

### Disks (disk_images)

Take the toml input and produce two files outputfilename_x.png and outputfilename_y.png

### process_file

Runs the toml and disk command together.

`cargo run --release -- test_data/line.png output -v1 -v1 -v1 -v1 process_file`

## tests

/test_data is what i am using durning devlopment. 

/test/images are code tests

mistakes are fun images from bad math.

# Does it work?

I dont know i am stuck on "find a friend with a laser cutter"

[1] https://www.drawmaton.com/
