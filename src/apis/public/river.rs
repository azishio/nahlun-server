use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use neo4rs::query;
use openapi::apis::public_river::{
    ApiRiverNodesBulkGetResponse, ApiRiverNodesGetResponse, PublicRiver,
};
use openapi::models::{
    ApiRiverNodesBulkGetQueryParams, ApiRiverNodesGetQueryParams,
};

use crate::apis::ServerImpl;
use crate::db;

impl PublicRiver for ServerImpl {
    async fn api_river_nodes_bulk_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiRiverNodesBulkGetQueryParams,
    ) -> Result<ApiRiverNodesBulkGetResponse, String> {
        let ApiRiverNodesBulkGetQueryParams {
            max_longitude,
            max_latitude,
            min_longitude,
            min_latitude,
        } = query_params;

        let query = query(
            r#"
                WITH
                    point({longitude:$max_longitude, latitude:$max_latitude}) AS upperRight,
                    point({longitude:$min_longitude, latitude:$min_latitude}) AS lowerLeft
                MATCH (n:RiverNode)
                    WHERE n.location IS NOT NULL AND
                    point.withinBBox(n.location, upperRight, lowerLeft)
                RETURN n
                "#,
        )
        .params([
            ("max_longitude", max_longitude),
            ("max_latitude", max_latitude),
            ("min_longitude", min_longitude),
            ("min_latitude", min_latitude),
        ]);

        let mut result = self.graph.execute(query).await.unwrap();

        let nodes = {
            let mut buf = Vec::new();
            while let Ok(Some(row)) = result.next().await {
                let node: db::RiverNode = row.get("n").unwrap();
                buf.push(node);
            }
            buf
        }
        .into_iter()
        .map(|node| node.into());

        Ok(ApiRiverNodesBulkGetResponse::Status200(nodes.collect()))
    }

    async fn api_river_nodes_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiRiverNodesGetQueryParams,
    ) -> Result<ApiRiverNodesGetResponse, String> {
        let ApiRiverNodesGetQueryParams { id, relation_limit } = query_params;

        let query = query(
            r#"
                MATCH path = (n:RiverNode{hilbert18:$id})-[:RIVER_LINK*0..$relation_limit]-(m:RiverNode)
                WITH path, nodes(path) as pathNodes
                ORDER BY size(relationships(path)) DESC

                WITH collect(pathNodes) as allPaths
                UNWIND allPaths as currentPath
                WITH currentPath, allPaths WHERE NONE(p IN allPaths WHERE size(p) > size(currentPath) AND apoc.coll.containsAll(p, currentPath))

                RETURN currentPath as path
                "#,
        );

        let mut result = self.graph.execute(query).await.unwrap();

        let nodes = {
            let mut buf = Vec::new();
            while let Ok(Some(row)) = result.next().await {
                let node = row.get::<Vec<db::RiverNode>>("path").unwrap();
                buf.push(node);
            }
            buf
        }
        .into_iter()
        .map(|nodes| nodes.into_iter().map(|node| node.into()).collect());

        Ok(ApiRiverNodesGetResponse::Status200(nodes.collect()))
    }
}
