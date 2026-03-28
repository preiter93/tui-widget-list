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
Sleep 1.5s
Show

# Classic
Sleep 1.5s
Right@0.5s 1
Down@0.2s 18

# Infinite
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.2s 20
Sleep 1.0s

# Scroll padding
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.2s 18
Up@0.2s 18

# Horizontal
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Down@0.5s 5

# Reversed
Left@1.5s 1
Down@1.5s 1

Right@0.5s 1
Up@0.2s 6
Down@0.2s 6

Sleep 1.5s
