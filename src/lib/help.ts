// Offline help: picks the built-in topic whose keywords best match the question.
export interface Topic {
  q: string
  keys: string[]
  answer: string
}

export const fold = (s: string) =>
  s
    .toLocaleLowerCase('tr')
    .replace(/ı/g, 'i')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^a-z0-9\s'-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()

// one typo (swap, missing or extra letter) is fine for longer words
function close(a: string, b: string) {
  if (Math.abs(a.length - b.length) > 1) return false
  let i = 0
  let j = 0
  let edits = 0
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      i++
      j++
      continue
    }
    if (++edits > 1) return false
    if (a.length > b.length) i++
    else if (b.length > a.length) j++
    else if (a[i + 1] === b[j] && a[i] === b[j + 1]) {
      i += 2
      j += 2
    } else {
      i++
      j++
    }
  }
  return edits + (a.length - i) + (b.length - j) <= 1
}

function matches(words: string[], text: string, key: string, exact: boolean) {
  if (key.includes(' ')) return text.includes(key)
  if (exact) return words.some((w) => w.startsWith(key))
  return words.some(
    (w) =>
      w.startsWith(key) ||
      (key.length >= 5 &&
        w.length >= key.length - 1 &&
        [key.length - 1, key.length, key.length + 1].some((n) => close(w.slice(0, n), key))),
  )
}

export function findTopic(question: string, topics: Topic[]): Topic | null {
  const text = fold(question)
  const words = text.split(' ')
  let best: Topic | null = null
  let bestScore = 0

  for (const topic of topics) {
    let score = 0
    for (const raw of topic.keys) {
      // "~" marks a generic word (change, use...) that only breaks ties,
      // "=" turns off typo matching (calismiyor vs calisiyor)
      const weak = raw.startsWith('~')
      const exact = raw.startsWith('=')
      const key = fold(weak || exact ? raw.slice(1) : raw)
      if (!key || !matches(words, text, key, exact)) continue
      score += weak ? 0.5 : key.includes(' ') || key.length > 4 ? 2 : 1.5
    }
    if (score > bestScore) {
      best = topic
      bestScore = score
    }
  }
  return bestScore >= 1 ? best : null
}
