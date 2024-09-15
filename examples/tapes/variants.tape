# To run this script, install vhs and run `vhs ./examples/tapes/variants.tape`
Output "examples/tapes/variants.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set Width 2000
Set Height 1100
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example variants"
Enter
Sleep 0.5s
Show

# Classic
Sleep 1.5s
Right@0.5s 1
Down@0.2s 40

# Infinite
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.2s 50
Sleep 1.0s

# Scroll padding
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.2s 40
Up@0.2s 40

# Horizontal
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.5s 10

Sleep 1.5s
