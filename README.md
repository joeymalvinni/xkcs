# XKCD Search Engine
Searches locally downloaded and indexed XKCD comics for given search queries.

<img width="1329" alt="image" src="https://github.com/joeymalvinni/xkcs/assets/76851062/6dcd5835-a080-43d4-b92e-ef791f3efb76">



# Todo:
- [ ] Reimplement fuzzy searching using substring matches instead of word vectorization (not accurate on alt text)
- [x] Lexographical sorting based on the secondary key of the title when ranks are the same
- [x] ~~Implement tries and DFS for autocomplete~~ (already tried, too slow)
- [x] Optimize HashMaps using non cryptographic hash methods
- [ ] ~~Add Levenschtein distance for similar results~~ (not fast enough)
