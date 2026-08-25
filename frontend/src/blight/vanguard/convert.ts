import type {
    VanguardBlockerResult,
    VanguardUrlSpecificResources,
    VanguardFilterListMetadata,
} from "vanguard";

export type SendableBlockerResult = {
    exception?: string;
    filter?: {
        raw_line?: string,
        source_location?: {
            line_number: number
            source_index: number
        }
    };
    important: boolean;
    redirect?: string;
    rewritten_url?: string;
}

export type SendableUrlSpecificResources = {
    exceptions: string[],
    generichide: boolean,
    hide_selectors: string[],
    procedural_actions: string[],
    injected_script: string,
};

export enum SendableExpiresIntervalType {
    Hours = 0,
    Days = 1
};

export type SendableFilterListMetadata = {
    expires?: { interval_type: SendableExpiresIntervalType, amount: number };
    homepage?: string;
    redirect?: string;
    title?: string;
  }

export type VanguardSendEntry = {
    type: string,
    url: string,
    bresult?: SendableBlockerResult,
    ureselt?: SendableUrlSpecificResources
}

export type IDBItem<T> = {
    key: string,
    value: T
}

export function resultToSendable(a: VanguardBlockerResult): SendableBlockerResult {
    return {
        important: a.important,
        ...a.filter && { 
            filter: {
                ...a.filter.raw_line && {raw_line: a.filter.raw_line}, 
                ...a.filter.source_location && {
                    source_location: {
                        line_number: a.filter.source_location.line_number,
                        source_index: a.filter.source_location.source_index
                    }
                }
            } 
        },
        ...a.redirect && {redirect: a.redirect},
        ...a.exception && a.exception.raw_line && {exception: a.exception.raw_line},
        ...a.rewritten_url && {rewritten_url: a.rewritten_url}
    };
}

export function urlResourcesToSendable(u: VanguardUrlSpecificResources): SendableUrlSpecificResources {
    return {
        exceptions: u.exceptions,
        generichide: u.generichide,
        hide_selectors: u.hide_selectors,
        procedural_actions: u.procedural_actions,
        injected_script: u.injected_script,
    }
}

export function filterListMetadataToSendable(l: VanguardFilterListMetadata): SendableFilterListMetadata  {
    return {
        ...l.expires && {
            expires: {
                interval_type: l.expires.interval_type as any,
                amount: l.expires.amount
            }
        },
        ...l.homepage && {homepage: l.homepage},
        ...l.redirect && {redirect: l.redirect},
        ...l.title && {title: l.title},
    }
}