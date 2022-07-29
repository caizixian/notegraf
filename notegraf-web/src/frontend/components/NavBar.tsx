import {Link, NavLink} from "react-router-dom";
import SearchBox from "./SearchBox";
import {DocumentAddIcon, LogoutIcon, MenuIcon} from "@heroicons/react/outline";
import * as React from "react";
import {useEffect, useRef, useState} from "react";

type MiscDropdownProps = {
    links: string[][]
}

function MiscDropdown(props: MiscDropdownProps) {
    const [open, setOpen] = useState(false);
    const ref = useRef<any>(null)
    useEffect(() => {
            const handleClickOutside = (event: MouseEvent) => {
                if (ref.current && !ref.current.contains(event.target)) {
                    setOpen(false);
                }
            };
            document.addEventListener('click', handleClickOutside, true);
            return () => {
                document.removeEventListener('click', handleClickOutside, true);
            };
        },
        []);

    return (
        <div className={"relative mr-2"} ref={ref}>
            <MenuIcon className={"h-6 w-6 inline"} onClick={() => {
                setOpen(!open);
            }}/>
            {
                open && (<div
                    className={"absolute bg-gray-500 mt-1.5 w-24 rounded-md divide-y divide-gray-300" +
                        "dark:divide-gray-700 shadow-lg ring-1 ring-white dark:ring-black ring-opacity-20 p-0.5"}>
                    {props.links.map(link => (
                        <div key={link[0]}>
                            <NavLink
                                to={link[0]}
                                onClick={() => {
                                    setOpen(false)
                                }}
                                end
                            >
                                {
                                    ({isActive}) => {
                                        const activeStyle = " bg-gray-400 dark:bg-gray-600";
                                        return (<div
                                            className={"px-2 py-2 block rounded-md" + (isActive ? activeStyle : "")}>
                                            {link[1]}
                                        </div>);
                                    }
                                }
                            </NavLink>
                        </div>
                    ))}
                </div>)
            }
        </div>
    );
}

export function NavBar() {
    // @ts-ignore
    let sso_signout_url = document.head.querySelector('meta[name="notegraf-sso-signout"]')!.content;

    return (
        <nav className={"flex p-1 w-full bg-gray-500 items-center gap-1"}>
            <MiscDropdown links={[["/note", "Recent"], ["/tags", "Tags"]]}/>
            <div className={"ml-auto flex gap-1 items-center min-w-0"}>
                <div className={"min-w-0"}>
                    <SearchBox/>
                </div>
                <Link to={"/note/new"}>
                    <button className={"ng-button ng-button-primary"}>
                        <DocumentAddIcon className={"h-6 w-6"}/>
                    </button>
                </Link>
                {sso_signout_url && <a href={sso_signout_url}>
                    <button className={"ng-button ng-button-primary"}>
                        <LogoutIcon className={"h-6 w-6"}/>
                    </button>
                </a>}
            </div>
        </nav>
    );
}