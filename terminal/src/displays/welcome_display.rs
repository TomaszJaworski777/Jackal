use utils::Theme;

pub fn welcome_message() -> String {
    let mut result = String::default();

    let logo: [String; 25] = [
        format!(
            "                   {}  .-=========-. {}                   ",
            "=.".secondary(0.0 / 24.0),
            ".=--".secondary(0.0 / 24.0)
        ),
        format!(
            "                 . {}===========:{}.                   ",
            "--.".secondary(1.0 / 24.0),
            ".:+=.-".secondary(1.0 / 24.0)
        ),
        format!(
            "              .:==.{}.==-.               ",
            "::-..:+=. .. :+=:. -.".secondary(2.0 / 24.0)
        ),
        format!(
            "            .=====-   {}======:             ",
            "+=...... ==:   -..".secondary(3.0 / 24.0)
        ),
        format!(
            "          .-======{}==-====:.          ",
            ":.:-=++:-:..--.  .:..:-".secondary(4.0 / 24.0)
        ),
        format!(
            "         :====={}=======.         ",
            "-..*++##=:::::.   .   .:.  -".secondary(5.0 / 24.0)
        ),
        format!(
            "        -===={}:=====:        ",
            ". +==:+=-.:::.::::::.    .:-==..".secondary(6.0 / 24.0)
        ),
        format!(
            "       -====={} .=: {}======:       ",
            "...-...".secondary(7.0 / 24.0),
            ".::::::..  ...:::  :-".secondary(7.0 / 24.0)
        ),
        format!(
            "      :======-{}:======-.      ",
            ".+--.  ...::::::...   ..::::-=.".secondary(8.0 / 24.0)
        ),
        format!(
            "     .---=-:{}-=-----      ",
            ".++-::::::...::::::.    ...:....-=.".secondary(9.0 / 24.0)
        ),
        format!(
            "     :--:{}-----.     ",
            ".-+:::-:::::::::::::..:.   ..::::. ..-:.".secondary(10.0 / 24.0)
        ),
        format!(
            "    .: {}      ----:     ",
            ".-: .::-::::.        ...   .. ..:::::".secondary(11.0 / 24.0)
        ),
        format!(
            "    .:{} -------:     ",
            ":  .::::::.            .. .  ...:.....:.".secondary(12.0 / 24.0)
        ),
        format!(
            "    .:--{}------:     ",
            ". ......------:    .    .. .:....... . .".secondary(13.0 / 24.0)
        ),
        format!(
            "    .:----{}------{} :-----.     ",
            ". :..".secondary(14.0 / 24.0),
            ":    ..:.  ...:::.:..    .".secondary(14.0 / 24.0)
        ),
        format!(
            "     .------------:{}----:.     ",
            "..  . ...::. :::::..:.       .".secondary(15.0 / 24.0)
        ),
        format!(
            "     .::-------:{}:::.      ",
            ".    ..  .:::..:-::....:.      :..".secondary(16.0 / 24.0)
        ),
        format!(
            "      .::::---:{}::::.       ",
            ".  ... .:---:.:--::..:....      .".secondary(17.0 / 24.0)
        ),
        format!(
            "       .:::::..:{}:::.       ",
            " ....:=--:..:-:....:. ..        .".secondary(18.0 / 24.0)
        ),
        format!(
            "        .:::::{}           ::..         ",
            ". ... :+--:....:.. ...".secondary(19.0 / 24.0)
        ),
        format!(
            "          .::{}::.          ",
            ".  ....:-::.....   ..        ..  .".secondary(20.0 / 24.0)
        ),
        format!(
            "           ..{}            ",
            ".   .. .:::...          .   ..  ...".secondary(21.0 / 24.0)
        ),
        format!(
            "                {}              ",
            ".    ..:.          ..   .   ..".secondary(22.0 / 24.0)
        ),
        format!("              . ...  ....  .    ...   ..                    ")
            .secondary(23.0 / 24.0),
        format!("                  ..   .. ..  ..     .                      ")
            .secondary(24.0 / 24.0),
    ];

    let logo: [String; 25] = {
        let mut result = std::array::from_fn(|_| String::new());

        for (idx, line) in logo.into_iter().enumerate() {
            result[idx] = line.primary(idx as f32 / 24.0)
        }

        result
    };

    let info: [String; 25] = {
        let mut result = std::array::from_fn(|_| String::new());

        result[4] = "Jackal Chess Engine".primary(4.0 / 24.0);
        result[6] = format!(
            "   {} {}",
            "Author: ".primary(6.0 / 24.0),
            "Tomasz Jaworski".secondary(6.0 / 24.0)
        );
        result[7] = format!(
            "   {} {}",
            "Version:".primary(7.0 / 24.0),
            env!("CARGO_PKG_VERSION").secondary(7.0 / 24.0)
        );

        result[11] = "Supported Non-UCI Commands".primary(11.0 / 24.0);
        result[13] = "   draw".secondary(13.0 / 24.0);
        result[14] = "   clear".secondary(14.0 / 24.0);
        result[15] = "   tree  <depth> <node_idx>".secondary(15.0 / 24.0);
        result[16] = "   perft <depth>".secondary(16.0 / 24.0);
        result[17] = "   bulk  <depth>".secondary(17.0 / 24.0);
        result[18] = "   bench <depth>".secondary(18.0 / 24.0);
        result[19] = "   analyse <nodes>".secondary(19.0 / 24.0);
        result[20] = "   eval".secondary(20.0 / 24.0);
        result[21] = "   policy".secondary(21.0 / 24.0);

        result
    };

    for (line, info) in logo.into_iter().zip(info) {
        result.push_str(format!("{line}    {info}\n").as_str());
    }

    result
}
