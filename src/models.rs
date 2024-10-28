use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LimelightResult {
    #[serde(default)]
    #[serde(rename = "Barcode")]
    pub barcode: Vec<BarcodeResult>,
    #[serde(default)]
    #[serde(rename = "Classifier")]
    pub classifier: Vec<ClassifierResult>,
    #[serde(default)]
    #[serde(rename = "Detector")]
    pub detector: Vec<DetectorResult>,
    #[serde(default)]
    #[serde(rename = "Fiducial")]
    pub fiducial: Vec<FiducialResult>,
    #[serde(default)]
    #[serde(rename = "Retro")]
    pub retro: Vec<ColorResult>,
    pub pipeline_type: Option<String>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub ta: Option<f64>,
    pub cl: Option<f64>,
    pub tl: Option<f64>,
    pub ts: Option<f64>,
    pub v: Option<f64>,
    pub focus_metric: Option<f64>,
    pub botpose: Option<Vec<f64>>,
    pub botpose_wpiblue: Option<Vec<f64>>,
    pub botpose_wpired: Option<Vec<f64>>,
    #[serde(rename = "botpose_orb")]
    pub botposeMT2: Option<Vec<f64>>,
    #[serde(rename = "botpose_orb_wpiblue")]
    pub botposeMT2_wpiblue: Option<Vec<f64>>,
    #[serde(rename = "botpose_orb_wpired")]
    pub botposeMT2_wpired: Option<Vec<f64>>,
    pub stdev_mt1: Option<Vec<f64>>,
    pub stdev_mt2: Option<Vec<f64>>,
    pub botpose_tagcount: Option<i32>,
    pub botpose_span: Option<f64>,
    pub botpose_avgdist: Option<f64>,
    pub botpose_avgarea: Option<f64>,
    pub python_out: Option<Vec<f64>>,
    pub txnc: Option<f64>,
    pub tync: Option<f64>,
    pub pipeline_id: Option<i32>,
    pub t6c_rs: Option<Vec<f64>>,
}

impl Default for LimelightResult {
    fn default() -> Self {
        Self {
            barcode: Vec::new(),
            classifier: Vec::new(),
            detector: Vec::new(),
            fiducial: Vec::new(),
            retro: Vec::new(),
            pipeline_type: None,
            tx: None,
            ty: None,
            ta: None,
            cl: None,
            tl: None,
            ts: None,
            v: None,
            focus_metric: None,
            botpose: None,
            botpose_wpiblue: None,
            botpose_wpired: None,
            botposeMT2: None,
            botposeMT2_wpiblue: None,
            botposeMT2_wpired: None,
            stdev_mt1: None,
            stdev_mt2: None,
            botpose_tagcount: None,
            botpose_span: None,
            botpose_avgdist: None,
            botpose_avgarea: None,
            python_out: None,
            txnc: None,
            tync: None,
            pipeline_id: None,
            t6c_rs: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BarcodeResult {
    pub fam: Option<String>,
    pub data: Option<String>,
    pub txp: Option<f64>,
    pub typ: Option<f64>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub tx_nocross: Option<f64>,
    pub ty_nocross: Option<f64>,
    pub ta: Option<f64>,
    pub pts: Option<Vec<Vec<f64>>>,
}

impl Default for BarcodeResult {
    fn default() -> Self {
        Self {
            fam: None,
            data: None,
            txp: None,
            typ: None,
            tx: None,
            ty: None,
            tx_nocross: None,
            ty_nocross: None,
            ta: None,
            pts: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassifierResult {
    pub class: Option<String>,
    #[serde(rename = "classID")]
    pub class_id: Option<i32>,
    pub conf: Option<f64>,
}

impl Default for ClassifierResult {
    fn default() -> Self {
        Self {
            class: None,
            class_id: None,
            conf: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetectorResult {
    pub class: Option<String>,
    #[serde(rename = "classID")]
    pub class_id: Option<i32>,
    pub conf: Option<f64>,
    pub ta: Option<f64>,
    pub txp: Option<f64>,
    pub typ: Option<f64>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub tx_nocross: Option<f64>,
    pub ty_nocross: Option<f64>,
    pub pts: Option<Vec<Vec<f64>>>,
}

impl Default for DetectorResult {
    fn default() -> Self {
        Self {
            class: None,
            class_id: None,
            conf: None,
            ta: None,
            txp: None,
            typ: None,
            tx: None,
            ty: None,
            tx_nocross: None,
            ty_nocross: None,
            pts: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FiducialResult {
    #[serde(rename = "fID")]
    pub f_id: Option<i32>,
    pub fam: Option<String>,
    pub skew: Option<Vec<f64>>,
    pub t6c_ts: Option<Vec<f64>>,
    pub t6r_fs: Option<Vec<f64>>,
    pub t6r_fs_orb: Option<Vec<f64>>,
    pub t6r_ts: Option<Vec<f64>>,
    pub t6t_cs: Option<Vec<f64>>,
    pub t6t_rs: Option<Vec<f64>>,
    pub ta: Option<f64>,
    pub txp: Option<f64>,
    pub typ: Option<f64>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub tx_nocross: Option<f64>,
    pub ty_nocross: Option<f64>,
    pub pts: Option<Vec<Vec<f64>>>,
}

impl Default for FiducialResult {
    fn default() -> Self {
        Self {
            f_id: None,
            fam: None,
            skew: None,
            t6c_ts: None,
            t6r_fs: None,
            t6r_fs_orb: None,
            t6r_ts: None,
            t6t_cs: None,
            t6t_rs: None,
            ta: None,
            txp: None,
            typ: None,
            tx: None,
            ty: None,
            tx_nocross: None,
            ty_nocross: None,
            pts: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorResult {
    pub t6c_ts: Option<Vec<f64>>,
    pub t6r_fs: Option<Vec<f64>>,
    pub t6r_ts: Option<Vec<f64>>,
    pub t6t_cs: Option<Vec<f64>>,
    pub t6t_rs: Option<Vec<f64>>,
    pub ta: Option<f64>,
    pub txp: Option<f64>,
    pub typ: Option<f64>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub tx_nocross: Option<f64>,
    pub ty_nocross: Option<f64>,
    pub pts: Option<Vec<Vec<f64>>>,
}

impl Default for ColorResult {
    fn default() -> Self {
        Self {
            t6c_ts: None,
            t6r_fs: None,
            t6r_ts: None,
            t6t_cs: None,
            t6t_rs: None,
            ta: None,
            txp: None,
            typ: None,
            tx: None,
            ty: None,
            tx_nocross: None,
            ty_nocross: None,
            pts: None,
        }
    }
}