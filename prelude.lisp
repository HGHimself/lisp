(def {fun} (\ {args body} {def (head args)   (\ (tail args) body)}))

(fun {list & xs} {xs})

(fun {cons x xs} {join (list x) xs})

(fun {first x : xs} {x})

(fun {second x : xs} {eval (head xs)})

(fun {curry f xs} {eval (join (list f) xs)})

(fun {uncurry f : xs} {f xs})

(fun {recur x : xs} {recur (tail (unpack xs))})
