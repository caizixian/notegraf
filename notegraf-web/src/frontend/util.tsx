import * as React from "react";
import {
    useLocation,
    useNavigate,
    useParams, useSearchParams,
} from "react-router-dom";

export type Empty = Record<any, never>;

export function withRouter(Component: any) {
    function ComponentWithRouterProp(props: any) {
        let location = useLocation();
        let navigate = useNavigate();
        let params = useParams();
        let [searchParams, setSearchParams] = useSearchParams();
        return (
            <Component
                {...props}
                router={{ location, navigate, params, searchParams, setSearchParams}}
            />
        );
    }
    return ComponentWithRouterProp;
}