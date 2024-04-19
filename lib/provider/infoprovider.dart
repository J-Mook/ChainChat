import 'package:chain_chat/messages/chatmessage.pb.dart';
import 'package:flutter/material.dart';

class InfoProvider with ChangeNotifier {

  bool isactivenotifier = false;
  void setnotifier(bool b) {
    isactivenotifier = b;
    // notifyListeners();
  }

  String name = "";
  void setName(String nnn) {
    if (nnn.isNotEmpty)
      name = nnn;
    notifyListeners();
  }

  List<MessagePair> messagesList = [];
  void addmessage(bool me, String name, String mssg, PColors ccc) {
    if(name.isNotEmpty && mssg.isNotEmpty)
    {
      Color pc = Color .fromARGB(255, (ccc.rrr / 2 + 128).toInt(), (ccc.bbb / 2 + 128).toInt(), (ccc.ggg / 2 + 128).toInt());
      messagesList.insert(0, MessagePair(me, name, mssg, pc));
    }

  }

}

class MessagePair {
  bool me;
  String name;
  String msg;
  Color ccc;

  MessagePair(this.me, this.name, this.msg, this.ccc);
}