fuse = null;

const search_input = document.querySelector(".search-input");
const search_list = document.querySelector(".search-list");
const search_results = document.querySelector(".search-results");

search_input.addEventListener("keyup", async function(event) {
  if (fuse === null) {
    fuse = await initSearch();
    console.log("Initializing Search");
  }

  search_list.innerHTML = "";

  const items = fuse.search(event.target.value);
  items.forEach((match) => {
    let element = document.createElement("li");
    element.innerHTML = searchItem(match);
    search_list.appendChild(element);
  })

  if (items.length > 0) {
    search_results.style.display = "block";
  } else {
    search_results.style.display = "none";
  }

});


async function initSearch() {
  return await fetch("/index.json").then(async function(response) {
    if (!response.ok) {
          throw new Error(`Response status: ${response.status}`);
    }
    const options = {
      keys: [
        "title",
        "html"
      ],
      includeMatches: true,
      minMatchCharLength: 3,
      /* findAllMatches: true */
      ignoreLocation: true
    };
    return new Fuse(await response.json(), options);
  });
}

function makeTeaser(match) {
  const [start, stop, largest] = largestMatch(match);
  if (largest.key == "title") {
    return [`<h1> ${largest.value} </h1>`, largest.value, ""];
  } else if (largest.key == "html"){
    const exact_match = largest.value.slice(start, stop + 1);
    const suffix = largest.value.slice(stop + 1).split(" ")[0].replace(/<[^>]*>?/gm, '');
    let teaser = largest.value.slice(Math.max(0, start - 60),start) 
      + '<b style="text-decoration: underline;">' 
      + exact_match
      + '</b>' 
      + largest.value.slice(stop + 1, Math.min(largest.value.length, stop + 61));
    return [teaser, exact_match, suffix];
  }
}

function largestMatch(item) {
  let max_length = 0;
  let start;
  let stop;
  let key_match;

  item.matches.forEach((key) => {
    for (let i = 0; i < key.indices.length; i++) {
      const length = key.indices[i][1] - key.indices[i][0] + 1;
      if (length > max_length) {
        max_length = length;
        start = key.indices[i][0];
        stop = key.indices[i][1];
        key_match = key;
      }
    }
  });

  return [start, stop, key_match]
}


function searchItem(match) {
  const [teaser, text, suffix] = makeTeaser(match);
  const suffix_part = suffix.length ? `,-${suffix}` : "";
  const url = encodeURI(`${match.item.url}#:~:text=${text}${suffix_part}`)
  return `<div class="search-item">`
    + `<a href="${url}">${match.item.title}</a>`
  + `<div>${teaser}</div>`
  + `</div>`

}
