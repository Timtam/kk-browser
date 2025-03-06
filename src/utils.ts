export const joinString = (
    s: string[],
    sep: string,
    final?: string,
): string => {
    if (!final) final = sep

    if (s.length === 1) return s[0]
    else if (s.length === 2) return s[0] + final + s[1]
    else return s[0] + sep + joinString(s.slice(1), sep, final)
}
