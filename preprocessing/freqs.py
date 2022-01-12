from os import path

from collections import defaultdict

dir = path.dirname(__file__)

with open(path.join(dir, "words.txt"), "r") as f:
    words = [word.lower().strip() for word in f.readlines()]

counts = [defaultdict(lambda: 0) for _ in range(0, 5)]
freqs = [{} for _ in range(0, 5)]

for i in range(0, 5):
    for word in words:
        counts[i][word[i]] += 1
    total_letters = sum(count for count in counts[i].values())
    freqs[i] = {
        char: count / total_letters
        for char, count in counts[i].items()
    }

print("[")
for i, freq in enumerate(freqs):
    print("[")
    for (c, f) in sorted(freq.items()):
        print(f"{f},")
    print("],")
print("]")