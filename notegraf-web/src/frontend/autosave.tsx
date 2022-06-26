import {debounce} from "lodash";
import * as React from "react";
import {useEffect, useState} from "react";

function getStorageValue(key: string, defaultValue: any): any {
    const saved = localStorage.getItem(key);
    if (saved) {
        const initial = JSON.parse(saved);
        return initial || defaultValue;
    } else {
        return defaultValue
    }
}

export function useLocalStorage(key: string, watch: any, setValue: any, defaultValue: any, debounceMS: number): any {
    const [value, _] = useState(() => {
        return getStorageValue(key, defaultValue);
    });

    const watchedValues = watch();

    useEffect(debounce(() => {
        localStorage.setItem(key, JSON.stringify(watchedValues));
    }, debounceMS), [key, watchedValues]);

    useEffect(() => {
            Object.keys(value).forEach((key) => {
                setValue(key, value[key], {
                    shouldValidate: true,
                    shouldDirty: true,
                    shouldTouch: true
                })
            })
        },
        []
    );
    return null;
}