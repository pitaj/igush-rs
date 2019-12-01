## push_front(x)

swap in new element
```
    0 1 2 3
0 [ c d|a b ] x
      ^       ^
1 [ h|e f g ]
2 [ j k l|i ]
3 [|m n     ]
```

move split
```
    0 1 2 3
0 [ c x|a b ]
     |<<
1 [ h|e f g ] d
2 [ j k l|i ]
3 [|m n     ]
```

swap into next row
```
    0 1 2 3
0 [ c|x a b ]
1 [ h|e f g ] d
    ^         ^
2 [ j k l|i ]
3 [|m n     ]
```

move split
```
    0 1 2 3
0 [ c|x a b ]
1 [ d|e f g ]
   |<<
2 [ j k l|i ] h
3 [|m n     ]
```

swap into next row
```
    0 1 2 3
0 [ c|x a b ]
1 [|d e f g ]
2 [ j k l|i ] h
        ^     ^
3 [|m n     ]
```

move split
```
    0 1 2 3
0 [ c|x a b ]
1 [|d e f g ]
2 [ j k h|i ]
       |<<
3 [|m n     ] l
```

insert at front of last DEQ
```
    0 1 2 3
0 [ c|x a b ]
1 [|d e f g ]
2 [ j k|h i ]
3 [|m n     ] l
   I
```

done
```
    0 1 2 3
0 [ c|x a b ]
1 [|d e f g ]
2 [ j k|h i ]
3 [|l m n   ]
```
