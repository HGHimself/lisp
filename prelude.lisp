def {true} 1
def {false} 0
def {nil} ()

(def {fun} (\ {args body} {def (head args)   (\ (tail args) body)}))

(fun {list & xs} {xs})

(fun {cons x xs} {join (list x) xs})
(fun {snoc x xs} {join xs (list x)})

(fun {first x : xs} {x})
(fun {second x : xs} {eval (head xs)})

(fun {curry f xs} {eval (join (list f) xs)})
(fun {uncurry f : xs} {f xs})

fun {len l} {if (empty l) {0} {+ 1 (len (tail l))}}

fun {reverse x} {if (== (len l) 1) {l} {snoc (head l) (reverse (tail l)) }}

fun {empty l} {if (== (len l) 0) {true} {false}}
fun {dec n} {- n 1}
fun {inc n} {+ n 1}

fun {nth l n} {if (== n 0) {head l} {nth (tail l) (dec n)}}

fun {do : l} {if (== l nil) {nil} {last l}}
