class MatchItem {
    regex: RegExp[];
    description: string;

    subMatches: MatchItem[] = [];

    constructor(regex: RegExp | RegExp[], desc: string, subs?: MatchItem | MatchItem[], caseSensitive: boolean = true) {
        if (regex instanceof RegExp) {
            regex = [regex];
        }
        this.regex = regex;
        this.description = desc;

        if (subs instanceof MatchItem) {
            this.subMatches.push(subs);
        } else if (subs instanceof Array) {
            this.subMatches = subs;
        }

        if (!caseSensitive) {
            this.regex = this.regex.map(r => new RegExp(r.source, r.flags + "i"));
        }
    }

    match(value: string) {
        for (const r of this.regex) {
            if (r.test(value)) {
                return true;
            }
        }
        return false;
    }

    generate(value: string): string[] {
        let ret = [this.description];
        for (const m of this.subMatches) {
            console.log(m, m.match(value));
            if (m.match(value)) {
                ret = ret.concat(m.generate(value));
            }
        }
        return ret;
    }
}



import regexStore from '@/assets/regex.json';

interface Raw {
    name: string;
    regex: string | string[];
    description: string;
    subMatches?: Raw[];
    caseSensitive?: boolean;
}

function loadRegex(regexStore: Raw[]): MatchItem[] {
    console.log(regexStore);
    return regexStore.map((r: Raw) => {
        const regex = r.regex instanceof Array
            ? r.regex.map((s: string) => new RegExp(s))
            : [new RegExp(r.regex)];
        const subs = r.subMatches ? loadRegex(r.subMatches) : undefined;
        const caseSensitive = r.caseSensitive ?? true;
        return new MatchItem(regex, r.description, subs, caseSensitive);
    });
}

const matchList: MatchItem[] = loadRegex(regexStore);

export function match(value: string): string[] | undefined {
    for (const m of matchList) {
        if (m.match(value)) {
            return m.generate(value);
        }
    }
    return undefined;
}