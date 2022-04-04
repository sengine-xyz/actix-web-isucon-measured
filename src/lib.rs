use std::future::{ready, Ready};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::default::Default;
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;

type DataType = Arc<RwLock<HashMap<(String, String), Vec<Duration>>>>;
#[derive(Debug, Clone, Default)]
pub struct Measured {
    pub mdata: DataType
}
pub enum SortOptions {
    PATH,
    METHOD,
    CNT,
    SUM,
    AVG,
    MAX,
    MIN,
}
impl Measured {
    pub fn tsv(&self, sortby: SortOptions) -> String {
        let mdata = self.mdata.read().unwrap();
        let mut v = vec![];
        for ((path, method), val) in (*mdata).iter() {
            let cnt = val.len();
            let sum = val.iter().sum::<Duration>().as_millis();
            let avg = sum / cnt as u128;
            let min = val.iter().min().unwrap().as_millis();
            let max = val.iter().max().unwrap().as_millis();
            v.push((path, method, cnt, sum, avg, max, min));
        }

        match sortby {
            SortOptions::PATH   => v.sort_by(|a, b| b.0.partial_cmp(a.0).unwrap()),
            SortOptions::METHOD => v.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap()),
            SortOptions::CNT    => v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap()),
            SortOptions::SUM    => v.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap()),
            SortOptions::AVG    => v.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap()),
            SortOptions::MAX    => v.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap()),
            SortOptions::MIN    => v.sort_by(|a, b| b.6.partial_cmp(&a.6).unwrap()),
        }

        let mut tsv = String::from("PATH\tMETHOD\tCNT\tSUM\tAVG\tMAX\tMIN\n");
        for (path, method, cnt, sum, avg, max, min) in v.iter() {
            tsv.push_str(format!("{path}\t{method}\t{cnt}\t{sum}\t{avg}\t{max}\t{min}\n").as_str());
        }
        tsv
    }

    pub fn clear(&self) {
        self.mdata.write().unwrap().clear();
    }
}

impl<S, B> Transform<S, ServiceRequest> for Measured
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MeasuredMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MeasuredMiddleware { 
            mdata: self.mdata.clone(),
            service
        }))
    }
}


pub struct MeasuredMiddleware<S> {
    mdata: DataType,
    service: S,
}

impl<S, B> Service<ServiceRequest> for MeasuredMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mdata = self.mdata.clone();
        let path = req.match_pattern().unwrap_or(req.path().to_string());
        let method = req.method().to_string();

        let fut = self.service.call(req);
        Box::pin(async move {
            let start = Instant::now();
            let res = fut.await?;
            mdata.write().unwrap().entry((path, method)).or_default().push(start.elapsed());
            Ok(res)
        })
    }
}


