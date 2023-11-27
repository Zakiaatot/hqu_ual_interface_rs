use crate::captcha_solver::captcha_solver;
use crate::crypto::encoded_password;
use anyhow::{anyhow, Result};
use reqwest::cookie::CookieStore;
use scraper::{Html, Selector};
use std::{collections::HashMap, sync::Arc};

pub async fn login(stu_num: String, password: String) -> Result<serde_json::Value> {
    let mut req = {
        match Req::new() {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow!("构造请求器失败: {}", e));
            }
        }
    };
    let (pwd_salt, execution) = {
        match req.get_ual_page().await {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow!("请求ual首页失败: {}", e));
            }
        }
    };
    let mut counter = 0;
    let mut may_err = None;
    loop {
        if counter == 3 {
            break;
        };
        match req.handle_captcha().await {
            Ok(_) => {
                break;
            }
            Err(e) => may_err = Some(e),
        }
        counter += 1;
    }
    if counter == 3 {
        return Err(anyhow!("处理验证码错误: {}", may_err.unwrap()));
    }
    match req.fake_login(stu_num, password, pwd_salt, execution).await {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow!("模拟登录错误: {}", e)),
    }
}

// config
const USER_AGENT:&str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.67";
const BASE_URL: &str = "http://id.hqu.edu.cn/authserver";

struct Req {
    client: reqwest::Client,
    cookie_jar: Arc<reqwest::cookie::Jar>,
}
impl Req {
    fn new() -> Result<Self> {
        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
        Ok(Req {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .cookie_provider(cookie_jar.clone())
                .user_agent(USER_AGENT)
                .build()?,
            cookie_jar,
        })
    }

    // return (pwdSalt,execution)
    async fn get_ual_page(&mut self) -> Result<(String, String)> {
        let html = self.client.get(BASE_URL.to_string() + "/login?service=https%3A%2F%2Fecard-ias.hqu.edu.cn%2Fias%2Fprelogin%3Fsysid%3DECARDSH").send().await?.text().await?;
        let document = Html::parse_document(&html);
        let selector = Selector::parse("#pwdEncryptSalt").unwrap();
        let execution_node;
        let pwd_salt = {
            if let Some(v) = document.select(&selector).next() {
                execution_node = v.next_sibling().unwrap();
                if let Some(v) = v.value().attr("value") {
                    v
                } else {
                    return Err(anyhow!("Get pwd_salt failed!"));
                }
            } else {
                return Err(anyhow!("Get pwd_salt failed!"));
            }
        };
        // println!("{}", pwd_salt);
        let execution = execution_node
            .value()
            .as_element()
            .unwrap()
            .attr("value")
            .unwrap();
        Ok((pwd_salt.to_string(), execution.to_string()))
    }

    async fn handle_captcha(&mut self) -> Result<()> {
        let res = self
            .client
            .get(BASE_URL.to_string() + "/common/openSliderCaptcha.htl")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let background = res
            .get("bigImage")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string();
        let slider = res
            .get("smallImage")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string();
        let answer = captcha_solver(slider, background, None)?;

        let mut form = HashMap::new();
        form.insert("canvasLength", "280");
        let binding = answer.to_string();
        form.insert("moveLength", binding.as_str());
        let res = self
            .client
            .post(BASE_URL.to_string() + "/common/verifySliderCaptcha.htl")
            .form(&form)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let res = res.get("errorCode").unwrap().to_string();
        if res == "1".to_string() {
            Ok(())
        } else {
            Err(anyhow!("Opencv failed"))
        }
    }

    async fn fake_login(
        &mut self,
        stu_num: String,
        password: String,
        pwd_salt: String,
        execution: String,
    ) -> Result<serde_json::Value> {
        let mut form = HashMap::new();
        form.insert("username", stu_num.clone());
        form.insert("password", encoded_password(password, pwd_salt)?);
        form.insert("captcha", "".to_string());
        form.insert("_eventId", "submit".to_string());
        form.insert("cllt", "userNameLogin".to_string());
        form.insert("dllt", "generalLogin".to_string());
        form.insert("lt", "".to_string());
        form.insert("execution", execution);

        let res = self.client.post(BASE_URL.to_string()+"/login?service=https%3A%2F%2Fecard-ias.hqu.edu.cn%2Fias%2Fprelogin%3Fsysid%3DECARDSH")
        .form(&form).send().await?;

        let mut json = serde_json::Map::new();

        // ual
        if let Some(v) = self.cookie_jar.cookies(
            &"http://id.hqu.edu.cn/authserver"
                .parse::<reqwest::Url>()
                .unwrap(),
        ) {
            let kv = v.to_str()?;
            for i in kv.split(';') {
                let j = i.trim();
                if j.starts_with("CASTGC=") {
                    let (_, v) = j.split_at(7);
                    json.insert(
                        "CASTGC".to_string(),
                        serde_json::Value::String(v.to_string()),
                    );
                }
            }
        }

        // failed
        if json.len() < 1 {
            let html = res.text().await?;
            let document = Html::parse_document(&html);
            let selector = Selector::parse("#pwdLoginDiv #showErrorTip span").unwrap();
            let error_msg = {
                match document.select(&selector).next() {
                    Some(v) => v.inner_html(),
                    None => "未知错误".to_string(),
                }
            };
            return Err(anyhow!(error_msg));
        }

        //ecard
        let mut form = HashMap::new();
        form.insert("errorcode", "1".to_string());
        form.insert("continueurl", "".to_string());
        form.insert("ssoticketid", stu_num);
        self.client
            .post("https://ecard-sh.hqu.edu.cn/cassyno/index")
            .form(&form)
            .send()
            .await?;

        //hallticket
        if let Some(v) = self.cookie_jar.cookies(
            &"http://ecard-sh.hqu.edu.cn/"
                .parse::<reqwest::Url>()
                .unwrap(),
        ) {
            let kv = v.to_str()?;
            for i in kv.split(';') {
                let j = i.trim();
                if j.starts_with("hallticket=") {
                    let (_, v) = j.split_at(11);
                    json.insert(
                        "hallticket".to_string(),
                        serde_json::Value::String(v.to_string()),
                    );
                }
            }
        }

        // failed
        if json.len() < 2 {
            return Err(anyhow!("get hallticket error"));
        }

        // gym
        let res = self
            .client
            .post("https://ecard-sh.hqu.edu.cn/Page/Page")
            .body("flowID=251&type=3&apptype=4&Url=https%253a%252f%252fecard-gymrsapp.hqu.edu.cn")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;
        let next = res.text().await?;
        if let None = next.split("window.location.href").last() {
            return Err(anyhow!("get ticket error"));
        };
        let next = next.split("window.location.href").last().unwrap();
        let mut next = next.split("'");
        next.next();
        let final_url;
        if let Some(url) = next.next() {
            final_url = url;
        } else {
            return Err(anyhow!("get ticket error"));
        }

        //finally
        self.client.get(final_url).send().await?;

        //JSESSIONID
        if let Some(v) = self.cookie_jar.cookies(
            &"http://ecard-gymrsapp.hqu.edu.cn/"
                .parse::<reqwest::Url>()
                .unwrap(),
        ) {
            let kv = v.to_str()?;
            for i in kv.split(';') {
                let j = i.trim();
                if j.starts_with("JSESSIONID=") {
                    let (_, v) = j.split_at(11);
                    json.insert(
                        "JSESSIONID".to_string(),
                        serde_json::Value::String(v.to_string()),
                    );
                }
            }
        }

        // failed
        if json.len() < 3 {
            return Err(anyhow!("get JSESSIONID error"));
        }

        Ok(serde_json::Value::Object(json))
    }
}
