# XKCD Search Engine
Searches locally downloaded and indexed XKCD comics for given search queries.


https://github.com/joeymalvinni/xkcs/assets/76851062/6c789a0a-6cea-452d-b139-a6730276d146


# Todo:
- [ ] Reimplement fuzzy searching using substring matches instead of word vectorization (not accurate on alt text)
- [x] Lexographical sorting based on the secondary key of the title when ranks are the same
- [x] ~~Implement tries and DFS for autocomplete~~ (already tried, too slow)
- [x] Optimize HashMaps using non cryptographic hash methods
- [ ] ~~Add Levenschtein distance for similar results~~ (not fast enough)
