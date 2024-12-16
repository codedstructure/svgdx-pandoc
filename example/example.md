## An Important Idea

Communicating important ideas can be effective when *text* and *diagrams* are combined.

```svgdx
<svg>
  <rect rx="2" surround="#w #d" margin="3" class="d-dot d-fill-lavender"/>
  <rect id="w" wh="20 10" text="Words" class="d-fill-lightblue"/>
  <ellipse id="d" xy="^:h 5" rxy="10 5" text="Diagrams" class="d-fill-khaki"/>
</svg>
```

This may increase both understanding and retention.

```svgdx
<svg>
  <rect rx="2" surround="#u1 #u2 #r1 #r2" margin="3" class="d-dot d-fill-lavender"/>
  <circle id="u1" xy="#u2:H 1" r="#u2 20%" class="d-fill-green"/>
  <circle id="u2" r="12" text="Understanding" class="d-fill-green d-text-italic"/>
  <rect id="r1" xy="#r2:H 1" wh="#r2 20%" class="d-fill-orchid"/>
  <rect id="r2" xy="#u2:h 10" wh="17 24" text="Retention" class="d-fill-orchid d-text-bold"/>
</svg>
```

Maybe try this in **your** next document!

```test
def fn():
  pass
```

``` test {x=1}
```

```
def ident(x):
  return x
```


Fin.
